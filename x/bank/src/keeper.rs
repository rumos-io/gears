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
use staking::BankKeeper as StakingBankKeeper;
use std::marker::PhantomData;
use std::{collections::HashMap, str::FromStr};

const SUPPLY_KEY: [u8; 1] = [0];
const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];
const DENOM_METADATA_PREFIX: [u8; 1] = [1];

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module> {
    store_key: SK,
    bank_params_keeper: BankParamsKeeper<PSK>,
    auth_keeper: AK,
    module_key: PhantomData<M>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module> BankKeeper<SK, M>
    for Keeper<SK, PSK, AK, M>
{
    fn send_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        from_address: AccAddress,
        to_module: &M,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        self.auth_keeper
            .check_create_new_module_account(ctx, to_module)?;

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

impl<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module>
    StakingBankKeeper<SK, M> for Keeper<SK, PSK, AK, M>
{
    fn get_all_balances<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: AccAddress,
    ) -> Result<Vec<Coin>, GasStoreErrors> {
        let bank_store = ctx.kv_store(&self.store_key);
        let prefix = create_denom_balance_prefix(addr);
        let account_store = bank_store.prefix_store(prefix);

        let mut balances = vec![];
        for rcoin in account_store.range(..) {
            let (_, coin) = rcoin?;
            let coin: Coin = Coin::decode::<Bytes>(coin.into_owned().into())
                .ok()
                .unwrap_or_corrupt();
            balances.push(coin);
        }
        Ok(balances)
    }

    /// send_coins_from_module_to_module delegates coins and transfers them from a
    /// delegator account to a module account. It creates the module accounts if it don't exist.
    /// It's safe operation because the modules are app generic parameter 
    /// which cannot be added in runtime.
    fn send_coins_from_module_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_pool: &M,
        recepient_pool: &M,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        self.auth_keeper
            .check_create_new_module_account(ctx, sender_pool)?;
        self.auth_keeper
            .check_create_new_module_account(ctx, recepient_pool)?;

        let msg = MsgSend {
            from_address: sender_pool.get_address(),
            to_address: recepient_pool.get_address(),
            amount,
        };

        self.send_coins(ctx, msg)
    }

    /// undelegate_coins_from_module_to_account undelegates the unbonding coins and transfers
    /// them from a module account to the delegator account. It will panic if the
    /// module account does not exist or is unauthorized.
    fn undelegate_coins_from_module_to_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_module: &M,
        addr: AccAddress,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        let sender_module_addr = sender_module.get_address();
        if self
            .auth_keeper
            .get_account(ctx, &sender_module_addr)?
            .is_none()
        {
            return Err(AppError::AccountNotFound);
        };
        if !sender_module
            .get_permissions()
            .iter()
            .any(|p| p == "staking")
        {
            return Err(AppError::Custom(format!(
                "module account {} does not have permissions to receive undelegate coins",
                sender_module.get_name()
            )));
        }
        self.undelegate_coins(ctx, sender_module_addr, addr, amount)
    }

    fn delegate_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_addr: AccAddress,
        recepient_module: &M,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        let recepient_module_addr = recepient_module.get_address();
        if self
            .auth_keeper
            .get_account(ctx, &recepient_module_addr)?
            .is_none()
        {
            return Err(AppError::AccountNotFound);
        };
        if !recepient_module
            .get_permissions()
            .iter()
            .any(|p| p == "staking")
        {
            return Err(AppError::Custom(format!(
                "module account {} does not have permissions to receive delegated coins",
                recepient_module.get_name()
            )));
        }
        self.delegate_coins(ctx, sender_addr, recepient_module_addr, amount)
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module>
    Keeper<SK, PSK, AK, M>
{
    pub fn new(store_key: SK, params_subspace_key: PSK, auth_keeper: AK) -> Self {
        let bank_params_keeper = BankParamsKeeper {
            params_subspace_key,
        };
        Keeper {
            store_key,
            bank_params_keeper,
            auth_keeper,
            module_key: PhantomData,
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

    // TODO: can we reuse with unwrap from `query_balance`?
    pub fn balance<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        address: &AccAddress,
        denom: &Denom,
    ) -> Result<Option<Coin>, GasStoreErrors> {
        let bank_store = ctx.kv_store(&self.store_key);
        let prefix = create_denom_balance_prefix(address.clone());

        let account_store = bank_store.prefix_store(prefix);
        let bal = account_store.get(denom.to_string().as_bytes())?;
        let res = bal.map(|bytes| {
            Coin::decode::<Bytes>(bytes.to_owned().into())
                .ok()
                .unwrap_or_corrupt()
        });
        Ok(res)
    }

    /// set_balance sets the coin balance for an account by address.
    pub fn set_balance<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        address: &AccAddress,
        amount: Coin,
    ) -> Result<(), GasStoreErrors> {
        let bank_store = ctx.kv_store_mut(&self.store_key);
        let prefix = create_denom_balance_prefix(address.clone());

        let mut account_store = bank_store.prefix_store_mut(prefix);
        if amount.amount.is_zero() {
            account_store.delete(amount.denom.to_string().as_bytes())?;
            Ok(())
        } else {
            account_store.set(
                amount.denom.to_string().as_bytes().to_vec(),
                amount.encode_vec().expect(IBC_ENCODE_UNWRAP),
            )
        }
    }

    /// add_coins increase the addr balance by the given amt. Fails if the provided amt is invalid.
    /// It emits a coin received event.
    pub fn add_coins<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        address: &AccAddress,
        amount: Vec<Coin>,
    ) -> Result<(), GasStoreErrors> {
        for coin in &amount {
            if let Some(mut balance) = self.balance(ctx, address, &coin.denom)? {
                balance.amount += coin.amount;
                self.set_balance(ctx, address, balance)?;
            } else {
                self.set_balance(ctx, address, coin.clone())?;
            }
        }

        // emit coin received event
        ctx.push_event(Event::new(
            "coin_received",
            [
                EventAttribute::new("receiver".into(), Vec::from(address.clone()).into(), true),
                // TODO: serialization of vector of coins
                EventAttribute::new(
                    "amount".into(),
                    serde_json::to_vec(&amount)
                        .unwrap_or(amount[0].encode_vec().expect(IBC_ENCODE_UNWRAP))
                        .into(),
                    true,
                ),
            ],
        ));
        Ok(())
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

    /// delegate_coins performs delegation by deducting amt coins from an account with
    /// address addr. For vesting accounts, delegations amounts are tracked for both
    /// vesting and vested coins. The coins are then transferred from the delegator
    /// address to a ModuleAccount address. If any of the delegation amounts are negative,
    /// an error is returned.
    fn delegate_coins<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegator_addr: AccAddress,
        module_acc_addr: AccAddress,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        if self
            .auth_keeper
            .get_account(ctx, &module_acc_addr)?
            .is_none()
        {
            return Err(AppError::AccountNotFound);
        };

        let mut balances = vec![];
        for coin in amount.inner() {
            if let Some(mut balance) = self.balance(ctx, &delegator_addr, &coin.denom)? {
                if balance.amount < coin.amount {
                    return Err(AppError::Custom(format!(
                        "failed to delegate; {} is smaller than {}",
                        balance.amount, coin.amount
                    )));
                }
                balances.push(balance.clone());
                balance.amount -= coin.amount;
                self.set_balance(ctx, &delegator_addr, balance)?;
            } else {
                return Err(AppError::Custom(format!(
                    "failed to delegate; 0 is smaller than {}",
                    coin.amount
                )));
            }
        }

        self.track_delegation(
            ctx,
            &delegator_addr,
            &SendCoins::new(balances.clone()).map_err(|e| AppError::Coins(e.to_string()))?,
            &amount,
        )?;

        // emit coin spent event
        ctx.push_event(Event::new(
            "coin_spent",
            [
                EventAttribute::new("spender".into(), Vec::from(delegator_addr).into(), true),
                // TODO: serialization of vector of coins
                EventAttribute::new(
                    "amount".into(),
                    serde_json::to_vec(&amount)
                        .unwrap_or(amount.inner()[0].encode_vec().expect(IBC_ENCODE_UNWRAP))
                        .into(),
                    true,
                ),
            ],
        ));

        Ok(self.add_coins(ctx, &module_acc_addr, balances)?)
    }

    /// undelegate_coins performs undelegation by crediting amt coins to an account with
    /// address addr. For vesting accounts, undelegation amounts are tracked for both
    /// vesting and vested coins. The coins are then transferred from a ModuleAccount
    /// address to the delegator address. If any of the undelegation amounts are
    /// negative, an error is returned.
    fn undelegate_coins<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        module_acc_addr: AccAddress,
        delegator_addr: AccAddress,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        if self
            .auth_keeper
            .get_account(ctx, &module_acc_addr)?
            .is_none()
        {
            return Err(AppError::AccountNotFound);
        };

        self.sub_unlocked_coins(ctx, &module_acc_addr, &amount)?;
        self.track_undelegation(ctx, &delegator_addr, &amount)?;
        Ok(self.add_coins(ctx, &delegator_addr, amount.into_inner())?)
    }

    /// sub_unlocked_coins removes the unlocked amt coins of the given account. An error is
    /// returned if the resulting balance is negative. A coin_spent event is emitted after.
    fn sub_unlocked_coins<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
        amount: &SendCoins,
    ) -> Result<(), AppError> {
        let locked_coins = self.locked_coins(ctx, addr)?;

        let amount_of = |coins: &Vec<Coin>, denom: &Denom| -> Uint256 {
            let coins = coins.iter().find(|c| c.denom == *denom);
            coins.map(|c| c.amount).unwrap_or(Uint256::zero())
        };

        for coin in amount.inner() {
            if let Some(mut balance) = self.balance(ctx, addr, &coin.denom)? {
                let locked_amount = amount_of(&locked_coins, &coin.denom);
                let spendable = balance.amount - locked_amount;

                if spendable.checked_sub(coin.amount).is_err() {
                    return Err(AppError::Coins(format!(
                        "{} is smaller than {}",
                        spendable, coin.amount
                    )));
                }

                balance.amount -= coin.amount;
                self.set_balance(ctx, addr, balance)?;
            } else {
                return Err(AppError::Coins(format!(
                    "Account {} doesn't have sufficient funds {}",
                    addr, &coin.denom
                )));
            }
        }

        // emit coin spent event
        ctx.push_event(Event::new(
            "coin_spent",
            [
                EventAttribute::new("spender".into(), Vec::from(addr.clone()).into(), true),
                // TODO: serialization of vector of coins
                EventAttribute::new(
                    "amount".into(),
                    serde_json::to_vec(&amount)
                        .unwrap_or(amount.inner()[0].encode_vec().expect(IBC_ENCODE_UNWRAP))
                        .into(),
                    true,
                ),
            ],
        ));
        Ok(())
    }

    /// locked_coins returns all the coins that are not spendable (i.e. locked) for an
    /// account by address. For standard accounts, the result will always be no coins.
    /// For vesting accounts, locked_coins is delegated to the concrete vesting account
    /// type.
    fn locked_coins<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
        // TODO: consider to add struct Coins that can have empty coins list
    ) -> Result<Vec<Coin>, AppError> {
        if let Some(_acc) = self.auth_keeper.get_account(ctx, addr)? {
            //     vacc, ok := acc.(vestexported.VestingAccount)
            //     if ok {
            //         return vacc.LockedCoins(ctx.BlockTime())
            //     }
            // TODO: logic with vesting accounts
            Ok(vec![])
        } else {
            Ok(vec![])
        }
    }

    /// track_delegation tracks the delegation of the given account if it is a vesting account
    fn track_delegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
        _balance: &SendCoins,
        _amount: &SendCoins,
    ) -> Result<(), AppError> {
        if let Some(_acc) = self.auth_keeper.get_account(ctx, addr)? {
            // TODO: logic with vesting accounts
            //     vacc, ok := acc.(vestexported.VestingAccount)
            //     if ok {
            //         vacc.TrackDelegation(ctx.BlockHeader().Time, balance, amt)
            //         k.ak.SetAccount(ctx, acc)
            //     }
            Ok(())
        } else {
            Err(AppError::AccountNotFound)
        }
    }

    /// track_undelegation trakcs undelegation of the given account if it is a vesting account
    fn track_undelegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &AccAddress,
        _amount: &SendCoins,
    ) -> Result<(), AppError> {
        if let Some(_acc) = self.auth_keeper.get_account(ctx, addr)? {
            // TODO: logic with vesting accounts
            //     vacc, ok := acc.(vestexported.VestingAccount)
            //     if ok {
            //         vacc.TrackUndelegation(amt)
            //         k.ak.SetAccount(ctx, acc)
            //     }
            Ok(())
        } else {
            Err(AppError::AccountNotFound)
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
