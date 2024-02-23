use std::{collections::HashMap, str::FromStr};

use bnum::types::U256;
use bytes::Bytes;
use database::Database;

use gears::types::context::context::Context;
use gears::types::context::init_context::InitContext;
use gears::types::context::query_context::QueryContext;
use gears::{
    baseapp::ante::AuthKeeper,
    error::AppError,
    x::{auth::Module, params::ParamsSubspaceKey},
};
use proto_messages::cosmos::ibc::protobuf::Protobuf;
use proto_messages::cosmos::{
    bank::v1beta1::{
        MsgSend, QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest,
        QueryBalanceResponse,
    },
    base::v1beta1::{Coin, SendCoins},
};
use proto_types::{AccAddress, Denom};
use store::{KVStore, MutablePrefixStore, StoreKey};
use tendermint::informal::abci::{Event, EventAttributeIndexExt};

use crate::{BankParamsKeeper, GenesisState};

const SUPPLY_KEY: [u8; 1] = [0];
const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    bank_params_keeper: BankParamsKeeper<SK, PSK>,
    auth_keeper: auth::Keeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> gears::baseapp::ante::BankKeeper<SK>
    for Keeper<SK, PSK>
{
    fn send_coins_from_account_to_module<DB: Database>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        from_address: AccAddress,
        to_module: Module,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        self.auth_keeper
            .check_create_new_module_account::<DB>(ctx, &to_module);

        let msg = MsgSend {
            from_address,
            to_address: to_module.get_address(),
            amount,
        };

        self.send_coins(ctx, msg)
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::x::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
        auth_keeper: auth::Keeper<SK, PSK>,
    ) -> Self {
        let bank_params_keeper = BankParamsKeeper {
            params_keeper,
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
        self.bank_params_keeper
            .set(&mut ctx.as_any(), genesis.params);

        let bank_store = ctx.get_mutable_kv_store(&self.store_key);

        let mut total_supply: HashMap<Denom, U256> = HashMap::new();
        for balance in genesis.balances {
            let prefix = create_denom_balance_prefix(balance.address);
            let mut denom_balance_store = bank_store.get_mutable_prefix_store(prefix);

            for coin in balance.coins {
                denom_balance_store.set(coin.denom.to_string().into_bytes(), coin.encode_vec());
                let zero = U256::ZERO;
                let current_balance = total_supply.get(&coin.denom).unwrap_or(&zero);
                total_supply.insert(coin.denom, coin.amount.0 + current_balance);
            }
        }

        // TODO: does the SDK sort these?
        for coin in total_supply {
            self.set_supply(
                &mut ctx.as_any(),
                Coin {
                    denom: coin.0,
                    amount: coin.1.into(),
                },
            );
        }
    }

    pub fn query_balance<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        req: QueryBalanceRequest,
    ) -> QueryBalanceResponse {
        let bank_store = ctx.get_kv_store(&self.store_key);
        let prefix = create_denom_balance_prefix(req.address);

        let account_store = bank_store.get_immutable_prefix_store(prefix);
        let bal = account_store.get(req.denom.to_string().as_bytes());

        match bal {
            Some(amount) => QueryBalanceResponse {
                balance: Some(
                    Coin::decode::<Bytes>(amount.to_owned().into())
                        .expect("invalid data in database - possible database corruption"),
                ),
            },
            None => QueryBalanceResponse { balance: None },
        }
    }

    pub fn query_all_balances<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        req: QueryAllBalancesRequest,
    ) -> QueryAllBalancesResponse {
        let bank_store = ctx.get_kv_store(&self.store_key);
        let prefix = create_denom_balance_prefix(req.address);
        let account_store = bank_store.get_immutable_prefix_store(prefix);

        let mut balances = vec![];

        for (_, coin) in account_store.range(..) {
            let coin: Coin = Coin::decode::<Bytes>(coin.to_owned().into())
                .expect("invalid data in database - possible database corruption");
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
        ctx: &QueryContext<'_, DB, SK>,
    ) -> Vec<Coin> {
        let bank_store = ctx.get_kv_store(&self.store_key);
        let supply_store = bank_store.get_immutable_prefix_store(SUPPLY_KEY.into());

        supply_store
            .range(..)
            .map(|raw_coin| {
                let denom = Denom::from_str(&String::from_utf8_lossy(&raw_coin.0))
                    .expect("invalid data in database - possible database corruption");
                let amount = U256::from_str(&String::from_utf8_lossy(&raw_coin.1))
                    .expect("invalid data in database - possible database corruption")
                    .into();
                Coin { denom, amount }
            })
            .collect()
    }

    pub fn send_coins_from_account_to_account<DB: Database>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        msg: &MsgSend,
    ) -> Result<(), AppError> {
        self.send_coins(ctx, msg.clone())?;

        // Create account if recipient does not exist

        if !self.auth_keeper.has_account(ctx, &msg.to_address) {
            self.auth_keeper
                .create_new_base_account(ctx, &msg.to_address);
        };

        Ok(())
    }

    fn send_coins<DB: Database>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        msg: MsgSend,
    ) -> Result<(), AppError> {
        // TODO: refactor this to subtract all amounts before adding all amounts

        let bank_store = ctx.get_mutable_kv_store(&self.store_key);
        let mut events = vec![];

        let from_address = msg.from_address;
        let to_address = msg.to_address;

        for send_coin in msg.amount {
            let mut from_account_store =
                Self::get_address_balances_store(bank_store, &from_address);
            let from_balance = from_account_store
                .get(send_coin.denom.to_string().as_bytes())
                .ok_or(AppError::Send("Insufficient funds".into()))?;

            let mut from_balance: Coin = Coin::decode::<Bytes>(from_balance.to_owned().into())
                .expect("invalid data in database - possible database corruption");

            if from_balance.amount < send_coin.amount {
                return Err(AppError::Send("Insufficient funds".into()));
            }

            from_balance.amount.0 -= send_coin.amount.0;

            from_account_store.set(
                send_coin.denom.clone().to_string().into(),
                from_balance.encode_vec(),
            );

            //TODO: if balance == 0 then denom should be removed from store

            let mut to_account_store = Self::get_address_balances_store(bank_store, &to_address);
            let to_balance = to_account_store.get(send_coin.denom.to_string().as_bytes());

            let mut to_balance: Coin = match to_balance {
                Some(to_balance) => Coin::decode::<Bytes>(to_balance.to_owned().into())
                    .expect("invalid data in database - possible database corruption"),
                None => Coin {
                    denom: send_coin.denom.clone(),
                    amount: U256::ZERO.into(),
                },
            };

            to_balance.amount.0 += send_coin.amount.0;

            to_account_store.set(send_coin.denom.to_string().into(), to_balance.encode_vec());

            events.push(Event::new(
                "transfer",
                vec![
                    ("recipient", String::from(to_address.clone())).index(),
                    ("sender", String::from(from_address.clone())).index(),
                    ("amount", send_coin.amount.to_string()).index(),
                ],
            ));
        }

        ctx.append_events(events);

        Ok(())
    }

    //#######

    pub fn set_supply<DB: Database>(&self, ctx: &mut Context<'_, '_, DB, SK>, coin: Coin) {
        // TODO: need to delete coins with zero balance

        let bank_store = ctx.get_mutable_kv_store(&self.store_key);
        let mut supply_store = bank_store.get_mutable_prefix_store(SUPPLY_KEY);

        supply_store.set(
            coin.denom.to_string().into(),
            coin.amount.to_string().into(),
        );
    }

    fn get_address_balances_store<'a, DB: Database>(
        bank_store: &'a mut KVStore<DB>,
        address: &AccAddress,
    ) -> MutablePrefixStore<'a, DB> {
        let prefix = create_denom_balance_prefix(address.to_owned());
        bank_store.get_mutable_prefix_store(prefix)
    }
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
