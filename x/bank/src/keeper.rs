use crate::types::query::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryDenomsMetadataResponse,
};
use crate::{BankParamsKeeper, GenesisState};
use bytes::Bytes;
use gears::context::{init::InitContext, query::QueryContext};
use gears::context::{QueryableContext, TransactionalContext};
use gears::error::{AppError, IBC_ENCODE_UNWRAP};
use gears::params::ParamsSubspaceKey;
use gears::store::database::ext::UnwrapCorrupt;
use gears::store::database::prefix::PrefixDB;
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::tendermint::types::proto::event::{Event, EventAttribute};
use gears::tendermint::types::proto::Protobuf;
use gears::types::address::AccAddress;
use gears::types::base::coin::Coin;
use gears::types::base::send::SendCoins;
use gears::types::denom::Denom;
use gears::types::msg::send::MsgSend;
use gears::types::store::gas::errors::GasStoreErrors;
use gears::types::store::prefix::mutable::PrefixStoreMut;
use gears::types::tx::metadata::Metadata;
use gears::types::uint::Uint256;
use gears::x::keepers::auth::AuthKeeper;
use gears::x::keepers::bank::BankKeeper;
use gears::x::module::Module;
use std::{collections::HashMap, str::FromStr};

const SUPPLY_KEY: [u8; 1] = [0];
const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];
const DENOM_METADATA_PREFIX: [u8; 1] = [1];

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK>> {
    store_key: SK,
    bank_params_keeper: BankParamsKeeper<PSK>,
    auth_keeper: AK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK>> BankKeeper<SK>
    for Keeper<SK, PSK, AK>
{
    fn send_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        from_address: AccAddress,
        to_module: Module,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        self.auth_keeper
            .check_create_new_module_account(ctx, &to_module)?;

        let msg = MsgSend {
            from_address,
            to_address: to_module.get_address(),
            amount,
        };

        self.send_coins(ctx, msg)
    }

    fn get_denom_metadata<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        base: &Denom,
    ) -> Result<Option<Metadata>, GasStoreErrors> {
        let bank_store = ctx.kv_store(&self.store_key);
        let denom_metadata_store = bank_store.prefix_store(denom_metadata_key(base.to_string()));

        Ok(denom_metadata_store
            .get(&base.to_string().into_bytes())?
            .map(|metadata| {
                Metadata::decode::<&[u8]>(&metadata)
                    .ok()
                    .unwrap_or_corrupt()
            }))
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK>> Keeper<SK, PSK, AK> {
    pub fn new(store_key: SK, params_subspace_key: PSK, auth_keeper: AK) -> Self {
        let bank_params_keeper = BankParamsKeeper {
            params_subspace_key,
        };
        Keeper {
            store_key,
            bank_params_keeper,
            auth_keeper,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        // TODO:
        // 1. cosmos SDK sorts the balances first
        // 2. Need to confirm that the SDK does not validate list of coins in each balance (validates order, denom etc.)
        // 3. Need to set denom metadata
        self.bank_params_keeper.set(ctx, genesis.params);

        let mut total_supply: HashMap<Denom, Uint256> = HashMap::new();
        for balance in genesis.balances {
            let prefix = create_denom_balance_prefix(balance.address);
            let mut denom_balance_store =
                ctx.kv_store_mut(&self.store_key).prefix_store_mut(prefix);

            for coin in balance.coins {
                denom_balance_store.set(
                    coin.denom.to_string().into_bytes(),
                    coin.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
                );
                let zero = Uint256::zero();
                let current_balance = total_supply.get(&coin.denom).unwrap_or(&zero);
                total_supply.insert(coin.denom, coin.amount + current_balance);
            }
        }

        // TODO: does the SDK sort these?
        for coin in total_supply {
            self.set_supply(
                ctx,
                Coin {
                    denom: coin.0,
                    amount: coin.1,
                },
            );
        }

        for denom_metadata in genesis.denom_metadata {
            self.set_denom_metadata(ctx, denom_metadata);
        }
    }

    pub fn query_balance<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        req: QueryBalanceRequest,
    ) -> QueryBalanceResponse {
        let bank_store = ctx.kv_store(&self.store_key);
        let prefix = create_denom_balance_prefix(req.address);

        let account_store = bank_store.prefix_store(prefix);
        let bal = account_store.get(req.denom.to_string().as_bytes());

        match bal {
            Some(amount) => QueryBalanceResponse {
                balance: Some(
                    Coin::decode::<Bytes>(amount.to_owned().into())
                        .ok()
                        .unwrap_or_corrupt(),
                ),
            },
            None => QueryBalanceResponse { balance: None },
        }
    }

    pub fn query_all_balances<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        req: QueryAllBalancesRequest,
    ) -> QueryAllBalancesResponse {
        let bank_store = ctx.kv_store(&self.store_key);
        let prefix = create_denom_balance_prefix(req.address);
        let account_store = bank_store.prefix_store(prefix);

        let mut balances = vec![];

        for (_, coin) in account_store.range(..) {
            let coin: Coin = Coin::decode::<Bytes>(coin.into_owned().into())
                .ok()
                .unwrap_or_corrupt();
            balances.push(coin);
        }

        QueryAllBalancesResponse {
            balances,
            pagination: None,
        }
    }

    /// Gets the total supply of every denom
    // TODO: should be paginated
    // TODO: should ignore coins with zero balance
    // TODO: does this method guarantee that coins are sorted?
    pub fn get_paginated_total_supply<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
    ) -> Vec<Coin> {
        let bank_store = ctx.kv_store(&self.store_key);
        let supply_store = bank_store.prefix_store(SUPPLY_KEY);

        supply_store
            .range(..)
            .map(|raw_coin| {
                let denom = Denom::from_str(&String::from_utf8_lossy(&raw_coin.0))
                    .ok()
                    .unwrap_or_corrupt();
                let amount = Uint256::from_str(&String::from_utf8_lossy(&raw_coin.1))
                    .ok()
                    .unwrap_or_corrupt();
                Coin { denom, amount }
            })
            .collect()
    }

    pub fn send_coins_from_account_to_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        msg: &MsgSend,
    ) -> Result<(), AppError> {
        self.send_coins(ctx, msg.clone())?;

        // Create account if recipient does not exist

        if !self.auth_keeper.has_account(ctx, &msg.to_address)? {
            self.auth_keeper
                .create_new_base_account(ctx, &msg.to_address)?;
        };

        Ok(())
    }

    fn send_coins<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        msg: MsgSend,
    ) -> Result<(), AppError> {
        // TODO: refactor this to subtract all amounts before adding all amounts

        let mut events = vec![];

        let from_address = msg.from_address;
        let to_address = msg.to_address;

        for send_coin in msg.amount {
            let mut from_account_store = self.get_address_balances_store(ctx, &from_address);
            let from_balance = from_account_store
                .get(send_coin.denom.to_string().as_bytes())?
                .ok_or(AppError::Send(format!(
                    "insufficient funds: required: {}, actual: 0",
                    send_coin.amount
                )))?;

            let mut from_balance: Coin = Coin::decode::<Bytes>(from_balance.to_owned().into())
                .ok()
                .unwrap_or_corrupt();

            if from_balance.amount < send_coin.amount {
                return Err(AppError::Send(format!(
                    "insufficient funds: required: {}, actual: {}",
                    send_coin.amount, from_balance.amount
                )));
            }

            from_balance.amount -= send_coin.amount;

            from_account_store.set(
                send_coin.denom.clone().to_string().into_bytes(),
                from_balance.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
            )?;

            //TODO: if balance == 0 then denom should be removed from store

            let mut to_account_store = self.get_address_balances_store(ctx, &to_address);
            let to_balance = to_account_store.get(send_coin.denom.to_string().as_bytes())?;

            let mut to_balance: Coin = match to_balance {
                Some(to_balance) => Coin::decode::<Bytes>(to_balance.to_owned().into())
                    .ok()
                    .unwrap_or_corrupt(),
                None => Coin {
                    denom: send_coin.denom.clone(),
                    amount: Uint256::zero(),
                },
            };

            to_balance.amount += send_coin.amount;

            to_account_store.set(
                send_coin.denom.to_string().into_bytes(),
                to_balance.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
            )?;

            events.push(Event::new(
                "transfer",
                [
                    EventAttribute::new(
                        "recipient".into(),
                        String::from(to_address.clone()).into(),
                        true,
                    ),
                    EventAttribute::new(
                        "sender".into(),
                        String::from(from_address.clone()).into(),
                        true,
                    ),
                    EventAttribute::new("amount".into(), send_coin.amount.to_string().into(), true),
                ],
            ));
        }

        ctx.append_events(events);

        Ok(())
    }

    pub fn set_supply<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, coin: Coin) {
        // TODO: need to delete coins with zero balance

        let bank_store = ctx.kv_store_mut(&self.store_key);
        let mut supply_store = bank_store.prefix_store_mut(SUPPLY_KEY);

        supply_store.set(
            coin.denom.to_string().into_bytes(),
            coin.amount.to_string().into_bytes(),
        );
    }

    fn get_address_balances_store<'a, DB: Database>(
        &'a self,
        ctx: &'a mut impl TransactionalContext<DB, SK>,
        address: &AccAddress,
    ) -> PrefixStoreMut<'a, PrefixDB<DB>> {
        let prefix = create_denom_balance_prefix(address.to_owned());
        let bank_store = ctx.kv_store_mut(&self.store_key);
        bank_store.prefix_store_mut(prefix)
    }

    /// Sets the denominations metadata
    pub fn set_denom_metadata<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        denom_metadata: Metadata,
    ) {
        // NOTE: we use the denom twice, once for the prefix and once for the key.
        // This seems unnecessary, I'm not sure why they do this in the SDK.
        let bank_store = ctx.kv_store_mut(&self.store_key);
        let mut denom_metadata_store =
            bank_store.prefix_store_mut(denom_metadata_key(denom_metadata.base.clone()));

        denom_metadata_store.set(
            denom_metadata.base.clone().into_bytes(),
            denom_metadata.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
        );
    }

    pub fn query_denoms_metadata<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
    ) -> QueryDenomsMetadataResponse {
        let bank_store = ctx.kv_store(&self.store_key);
        let mut denoms_metadata = vec![];

        for (_, metadata) in bank_store.prefix_store(DENOM_METADATA_PREFIX).range(..) {
            let metadata: Metadata = Metadata::decode::<Bytes>(metadata.into_owned().into())
                .ok()
                .unwrap_or_corrupt();
            denoms_metadata.push(metadata);
        }

        QueryDenomsMetadataResponse {
            metadatas: denoms_metadata,
            pagination: None,
        }
    }
}

fn denom_metadata_key(denom: String) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend(DENOM_METADATA_PREFIX);
    key.extend(denom.into_bytes());
    key
}

fn create_denom_balance_prefix(addr: AccAddress) -> Vec<u8> {
    let addr_len = addr.len();
    let mut addr: Vec<u8> = addr.into();
    let mut prefix = Vec::new();

    prefix.extend(ADDRESS_BALANCES_STORE_PREFIX);
    prefix.push(addr_len);
    prefix.append(&mut addr);

    prefix
}

//TODO: copy tests across
