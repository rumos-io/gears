use crate::{
    BondStatus, Delegation, DvPair, DvvTriplet, GenesisState, LastValidatorPower, Pool,
    Redelegation, StakingParamsKeeper, UnbondingDelegation, Validator,
};
use chrono::Utc;
use gears::{
    core::address::{AccAddress, ValAddress},
    crypto::keys::ReadAccAddress,
    error::AppError,
    params::ParamsSubspaceKey,
    store::{
        database::Database, QueryableKVStore, ReadPrefixStore, StoreKey, TransactionalKVStore,
        WritePrefixStore,
    },
    tendermint::types::proto::{
        event::{Event, EventAttribute},
        validator::ValidatorUpdate,
    },
    types::{
        base::{coin::Coin, send::SendCoins},
        context::{init::InitContext, QueryableContext, TransactionalContext},
        decimal256::Decimal256,
        uint::Uint256,
    },
    x::keepers::auth::AuthKeeper,
};
use prost::{bytes::BufMut, Message};
use serde::de::Error;
use std::{cmp::Ordering, collections::HashMap};

/// AccountKeeper defines the expected account keeper methods (noalias)
// TODO: AuthKeeper should implements module account stuff
pub trait AccountKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    // TODO: should be a sdk account interface
    fn get_account<DB: Database, AK: AuthKeeper<SK>, CTX: QueryableContext<DB, SK>>(
        _ctx: CTX,
        _addr: ValAddress,
    ) -> AK;

    // only used for simulation
    fn get_module_address(_name: String) -> ValAddress {
        todo!()
    }

    fn get_module_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        _ctx: &CTX,
        _module_name: String,
    ) -> Self;

    fn set_module_account<DB: Database, AK: AuthKeeper<SK>, CTX: QueryableContext<DB, SK>>(
        _context: &CTX,
        _acc: AK,
    );
}

/// BankKeeper defines the expected interface needed to retrieve account balances.
pub trait BankKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    // GetAllBalances(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    // GetBalance(ctx sdk.Context, addr sdk.AccAddress, denom string) sdk.Coin
    // LockedCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    // SpendableCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    //
    // GetSupply(ctx sdk.Context, denom string) sdk.Coin
    //
    // BurnCoins(ctx sdk.Context, name string, amt sdk.Coins) error

    fn send_coins_from_module_to_module<
        DB: Database,
        AK: AccountKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        sender_pool: String,
        recepient_pool: String,
        amount: SendCoins,
    ) -> Result<(), AppError>;

    fn undelegate_coins_from_module_to_account<
        DB: Database,
        AK: AccountKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        sender_module: String,
        addr: AccAddress,
        amount: SendCoins,
    ) -> Result<(), AppError>;

    fn delegate_coins_from_account_to_module<
        DB: Database,
        AK: AccountKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        sender_addr: AccAddress,
        recepient_module: String,
        amount: SendCoins,
    ) -> Result<(), AppError>;
}

/// Event Hooks
/// These can be utilized to communicate between a staking keeper and another
/// keeper which must take particular actions when validators/delegators change
/// state. The second keeper must implement this interface, which then the
/// staking keeper can call.
pub trait KeeperHooks<SK: StoreKey>: Clone + Send + Sync + 'static {
    fn after_validator_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
    );

    fn before_validator_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
    );

    fn after_validator_removed<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        // TODO: ConstAddr in cosmos sdk
        const_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn after_validator_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        // TODO: ConstAddr in cosmos sdk
        const_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn after_validator_begin_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        // TODO: ConstAddr in cosmos sdk
        const_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_delegation_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_delegation_shares_modified<
        DB: Database,
        AK: AuthKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_delegation_removed<
        DB: Database,
        AK: AuthKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn after_delegation_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_validator_slashed<
        DB: Database,
        AK: AuthKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
        // TODO: original is an alias to bigint
        fraction: Decimal256,
    );
}

const POOL_KEY: [u8; 1] = [0];
const LAST_TOTAL_POWER_KEY: [u8; 1] = [1];
const VALIDATORS_KEY: [u8; 1] = [2];
const LAST_VALIDATOR_POWER_KEY: [u8; 1] = [3];
const DELEGATIONS_KEY: [u8; 1] = [4];
pub(crate) const VALIDATORS_BY_POWER_INDEX_KEY: [u8; 1] = [4];
const VALIDATORS_QUEUE_KEY: [u8; 1] = [5];
const UBD_QUEUE_KEY: [u8; 1] = [6];
const UNBONDING_QUEUE_KEY: [u8; 1] = [7];
const REDELEGATION_QUEUE_KEY: [u8; 1] = [8];

const NOT_BONDED_POOL_NAME: &str = "not_bonded_tokens_pool";
const BONDED_POOL_NAME: &str = "bonded_tokens_pool";
const EVENT_TYPE_COMPLETE_UNBONDING: &str = "complete_unbonding";
const EVENT_TYPE_COMPLETE_REDELEGATION: &str = "complete_redelegation";
const ATTRIBUTE_KEY_AMOUNT: &str = "amount";
const ATTRIBUTE_KEY_VALIDATOR: &str = "validator";
const ATTRIBUTE_KEY_DELEGATOR: &str = "delegator";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Keeper<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AccountKeeper<SK>,
    BK: BankKeeper<SK>,
    KH: KeeperHooks<SK>,
> {
    store_key: SK,
    auth_keeper: AK,
    bank_keeper: BK,
    staking_params_keeper: StakingParamsKeeper<SK, PSK>,
    codespace: String,
    hooks_keeper: Option<KH>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn new(
        store_key: SK,
        params_subspace_key: PSK,
        auth_keeper: AK,
        bank_keeper: BK,
        params_keeper: gears::params::Keeper<SK, PSK>,
        codespace: String,
    ) -> Self {
        let staking_params_keeper = StakingParamsKeeper {
            params_keeper,
            params_subspace_key,
        };

        Keeper {
            store_key,
            auth_keeper,
            bank_keeper,
            staking_params_keeper,
            codespace,
            hooks_keeper: None,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) -> anyhow::Result<()> {
        // TODO
        // ctx = ctx.WithBlockHeight(1 - sdk.ValidatorUpdateDelay)

        self.set_pool(ctx, genesis.pool)?;
        self.set_last_total_power(ctx, genesis.last_total_power);
        self.staking_params_keeper
            .set(&mut ctx.multi_store_mut(), genesis.params)?;

        for validator in genesis.validators {
            self.set_validator(ctx, &validator)?;
            self.set_validator_by_cons_addr(ctx, &validator)?;
            self.set_validator_by_power_index(ctx, &validator)?;

            if !genesis.exported {
                self.after_validator_created(ctx, &validator)?;
            }

            if validator.status == BondStatus::Unbonding {
                // TODO
                //     self.insert_validator_queue(ctx, &validator)
            }
        }

        for delegation in genesis.delegations {
            if !genesis.exported {
                self.before_delegation_created(ctx, &delegation)?;
            }

            self.set_delegation(ctx, &delegation)?;

            if !genesis.exported {
                self.after_delegation_modified(ctx, &delegation)?;
            }
        }

        for unbonding_delegation in genesis.unbonding_delegations {
            self.set_unbonding_delegation(ctx, &unbonding_delegation)?;
            for entry in unbonding_delegation.entries.as_slice() {
                self.insert_ubd_queue(ctx, &unbonding_delegation, entry.completion_time)?;
            }
        }

        for redelegation in genesis.redelegations {
            self.set_redelegation(ctx, &redelegation)?;
            for _entry in redelegation.entries {
                // TODO
                // self.insert_redelegation_queue(ctx, &redelegation, entry.completion_time)?;
            }
        }
        // // don't need to run Tendermint updates if we exported
        if genesis.exported {
            for last_validator in genesis.last_validator_powers {
                self.set_last_validator_power(ctx, &last_validator)?;
                // let validator =
                //     self.get_validator(ctx, &last_validator.address.to_string().as_bytes())?;
                // let mut update = validator.abci_validator_update();
                // update.1 = last_validator.power;
            }
            Ok(())
        } else {
            // TODO
            // self.apply_and_return_validator_set_updates(ctx)?;
            Ok(())
        }
    }

    pub fn set_pool<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        pool: Pool,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut pool_store = store.prefix_store_mut(POOL_KEY);
        let pool = serde_json::to_vec(&pool)?;
        pool_store.set(pool.clone(), pool);
        Ok(())
    }

    /// Load the last total validator power.
    pub fn get_last_total_power<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Option<Uint256> {
        let store = ctx.kv_store(&self.store_key);
        store.get(&LAST_TOTAL_POWER_KEY).map(|bytes| {
            Uint256::from_be_bytes(bytes.try_into().expect("Unexpected conversion error."))
        })
    }

    pub fn set_last_total_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        last_total_power: Uint256,
    ) {
        let mut store = ctx.kv_store_mut(&self.store_key);
        store.set(LAST_TOTAL_POWER_KEY, last_total_power.to_be_bytes());
    }

    pub fn get_validator<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        key: &[u8],
    ) -> anyhow::Result<Validator> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(VALIDATORS_KEY);
        if let Some(e) = validators_store.get(key) {
            Ok(serde_json::from_slice(&e)?)
        } else {
            Err(anyhow::Error::from(serde_json::Error::custom(
                "Validator doesn't exists.".to_string(),
            )))
        }
    }

    pub fn set_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.set(
            validator.operator_address.to_string().as_bytes().to_vec(),
            serde_json::to_vec(&validator)?,
        );
        Ok(())
    }

    pub fn remove_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &[u8],
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.delete(addr);
        Ok(())
    }

    /// get the last validator set
    pub fn get_last_validators_by_addr<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> anyhow::Result<HashMap<String, Vec<u8>>> {
        let mut last = HashMap::new();
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(LAST_VALIDATOR_POWER_KEY);
        for (k, v) in store.range(..) {
            let k: ValAddress = serde_json::from_slice(&k)?;
            last.insert(k.to_string(), v.to_vec());
        }
        Ok(last)
    }

    /// get the last validator set
    // TODO: is a hack that allows to use store in the code after call,
    // Otherwise, it borrows the store and it cannot be reused in mutable calls
    pub fn get_validators_power_store_vals_map<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> anyhow::Result<HashMap<Vec<u8>, ValAddress>> {
        let store = ctx.kv_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATORS_BY_POWER_INDEX_KEY);
        let mut res = HashMap::new();
        for (k, v) in iterator.range(..) {
            res.insert(k.to_vec(), serde_json::from_slice(&v)?);
        }
        Ok(res)
    }

    pub fn set_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        let power_reduction = self.power_reduction(ctx);
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_BY_POWER_INDEX_KEY);

        // jailed validators are not kept in the power index
        if validator.jailed {
            return Ok(());
        }

        validators_store.set(
            validator.key_by_power_index_key(power_reduction),
            validator.operator_address.to_string().as_bytes().to_vec(),
        );
        Ok(())
    }

    pub fn delete_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Option<Vec<u8>> {
        let power_reduction = self.power_reduction(ctx);
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(VALIDATORS_BY_POWER_INDEX_KEY);
        store.delete(&validator.key_by_power_index_key(power_reduction))
    }

    pub fn set_validator_by_cons_addr<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.set(
            validator
                .consensus_pubkey
                .get_address()
                .to_string()
                .encode_to_vec(),
            serde_json::to_vec(&validator)?,
        );
        Ok(())
    }

    pub fn set_delegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Delegation,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(DELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_address.to_string().as_bytes());
        delegations_store.set(key, serde_json::to_vec(&delegation)?);
        Ok(())
    }

    pub fn get_unbonding_delegation<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    ) -> Option<UnbondingDelegation> {
        let store = ctx.kv_store(&self.store_key);
        let delegations_store = store.prefix_store(DELEGATIONS_KEY);
        let mut key = del_addr.to_string().as_bytes().to_vec();
        key.put(val_addr.to_string().as_bytes());
        if let Some(bytes) = delegations_store.get(&key) {
            if let Ok(delegation) = serde_json::from_slice(&bytes) {
                return Some(delegation);
            }
        }
        None
    }

    pub fn set_unbonding_delegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &UnbondingDelegation,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(DELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_address.to_string().as_bytes());
        delegations_store.set(key, serde_json::to_vec(&delegation)?);
        Ok(())
    }

    pub fn remove_unbonding_delegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &UnbondingDelegation,
    ) -> Option<Vec<u8>> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(DELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_address.to_string().as_bytes());
        delegations_store.delete(&key)
    }

    /// Returns a concatenated list of all the timeslices inclusively previous to
    /// currTime, and deletes the timeslices from the queue
    pub fn dequeue_all_mature_redelegation_queue<
        DB: Database,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        time: chrono::DateTime<Utc>,
    ) -> anyhow::Result<Vec<DvvTriplet>> {
        let (keys, mature_redelegations) = {
            let storage = ctx.kv_store(&self.store_key);
            let store = storage.prefix_store(REDELEGATION_QUEUE_KEY);

            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            let end = {
                let mut k = get_unbonding_delegation_time_key(time);
                k.push(0);
                k
            };
            let mut mature_redelegations = vec![];
            let mut keys = vec![];
            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            for (k, v) in store.range(..).take_while(|(k, _)| **k != end) {
                let time_slice: Vec<DvvTriplet> = serde_json::from_slice(&v)?;
                mature_redelegations.extend(time_slice);
                keys.push(k.to_vec());
            }
            (keys, mature_redelegations)
        };

        let storage = ctx.kv_store_mut(&self.store_key);
        let mut store = storage.prefix_store_mut(UNBONDING_QUEUE_KEY);
        keys.iter().for_each(|k| {
            store.delete(k);
        });
        Ok(mature_redelegations)
    }

    /// Returns a concatenated list of all the timeslices inclusively previous to
    /// currTime, and deletes the timeslices from the queue
    pub fn dequeue_all_mature_ubd_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: chrono::DateTime<Utc>,
    ) -> anyhow::Result<Vec<DvPair>> {
        let (keys, mature_unbonds) = {
            let storage = ctx.kv_store(&self.store_key);
            let store = storage.prefix_store(UNBONDING_QUEUE_KEY);
            let end = {
                let mut k = get_unbonding_delegation_time_key(time);
                k.push(0);
                k
            };
            let mut mature_unbonds = vec![];
            let mut keys = vec![];
            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            for (k, v) in store.range(..).take_while(|(k, _)| **k != end) {
                let time_slice: Vec<DvPair> = serde_json::from_slice(&v)?;
                mature_unbonds.extend(time_slice);
                keys.push(k.to_vec());
            }
            (keys, mature_unbonds)
        };
        let storage = ctx.kv_store_mut(&self.store_key);
        let mut store = storage.prefix_store_mut(UNBONDING_QUEUE_KEY);
        keys.iter().for_each(|k| {
            store.delete(k);
        });
        Ok(mature_unbonds)
    }

    /// Insert an unbonding delegation to the appropriate timeslice in the unbonding queue
    pub fn insert_ubd_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &UnbondingDelegation,
        time: chrono::DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let time_slice = self.get_ubd_queue_time_slice(ctx, time);
        let dv_pair = DvPair::new(
            delegation.validator_address.clone(),
            delegation.delegator_address.clone(),
        );

        if let Some(mut time_slice) = time_slice {
            time_slice.push(dv_pair);
            self.set_ubd_queue_time_slice(ctx, time, time_slice)
        } else {
            self.set_ubd_queue_time_slice(ctx, time, vec![dv_pair])
        }
    }

    pub fn insert_unbonding_validator_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        let mut addrs = self.get_unbonding_validators(
            ctx,
            validator.unbonding_time,
            validator.unbonding_height,
        )?;
        addrs.push(validator.operator_address.to_string());
        self.set_unbonding_validators_queue(
            ctx,
            validator.unbonding_time,
            validator.unbonding_height,
            addrs,
        )
    }

    pub fn get_ubd_queue_time_slice<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: chrono::DateTime<Utc>,
    ) -> Option<Vec<DvPair>> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(UBD_QUEUE_KEY);
        if let Some(bz) = store.get(time.to_string().as_bytes()) {
            serde_json::from_slice(&bz).unwrap_or_default()
        } else {
            None
        }
    }

    pub fn set_ubd_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: chrono::DateTime<Utc>,
        time_slice: Vec<DvPair>,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(UBD_QUEUE_KEY);
        let key = time.to_string().as_bytes().to_vec();
        store.set(key, serde_json::to_vec(&time_slice)?);
        Ok(())
    }

    pub fn get_redelegation<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_src_addr: ValAddress,
        val_dst_addr: ValAddress,
    ) -> anyhow::Result<Redelegation> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(DELEGATIONS_KEY);
        let mut key = del_addr.to_string().as_bytes().to_vec();
        key.put(val_src_addr.to_string().as_bytes());
        key.put(val_dst_addr.to_string().as_bytes());
        if let Some(e) = store.get(&key) {
            Ok(serde_json::from_slice(&e)?)
        } else {
            Err(anyhow::Error::from(serde_json::Error::custom(
                "Validator doesn't exists.".to_string(),
            )))
        }
    }

    pub fn set_redelegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Redelegation,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(DELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_src_address.to_string().as_bytes());
        key.put(delegation.validator_dst_address.to_string().as_bytes());
        delegations_store.set(key, serde_json::to_vec(&delegation)?);
        Ok(())
    }

    pub fn remove_redelegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Redelegation,
    ) -> Option<Vec<u8>> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(DELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_src_address.to_string().as_bytes());
        key.put(delegation.validator_dst_address.to_string().as_bytes());
        delegations_store.delete(&key)
    }

    pub fn set_last_validator_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &LastValidatorPower,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(LAST_VALIDATOR_POWER_KEY);
        let key = validator.address.to_string().as_bytes().to_vec();
        delegations_store.set(key, serde_json::to_vec(&validator)?);
        Ok(())
    }

    pub fn delete_last_validator_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &ValAddress,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(LAST_VALIDATOR_POWER_KEY);
        delegations_store.delete(validator.to_string().as_bytes());
        Ok(())
    }

    pub fn after_validator_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_validator_created(ctx, validator.operator_address.clone());
        }
        Ok(())
    }

    pub fn before_delegation_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Delegation,
    ) -> anyhow::Result<()> {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.before_delegation_created(
                ctx,
                delegation.delegator_address.clone(),
                delegation.validator_address.clone(),
            );
        }
        Ok(())
    }

    pub fn after_delegation_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Delegation,
    ) -> anyhow::Result<()> {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_delegation_modified(
                ctx,
                delegation.delegator_address.clone(),
                delegation.validator_address.clone(),
            );
        }
        Ok(())
    }

    pub fn after_validator_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_validator_bonded(
                ctx,
                validator.get_cons_addr(),
                validator.operator_address.clone(),
            );
        }
        Ok(())
    }

    pub fn after_validator_begin_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_validator_begin_unbonding(
                ctx,
                validator.get_cons_addr(),
                validator.operator_address.clone(),
            );
        }
        Ok(())
    }

    /// BlockValidatorUpdates calculates the ValidatorUpdates for the current block
    /// Called in each EndBlock
    pub fn block_validator_updates<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> anyhow::Result<Vec<ValidatorUpdate>> {
        // Calculate validator set changes.

        // NOTE: ApplyAndReturnValidatorSetUpdates has to come before
        // UnbondAllMatureValidatorQueue.
        // This fixes a bug when the unbonding period is instant (is the case in
        // some of the tests). The test expected the validator to be completely
        // unbonded after the Endblocker (go from Bonded -> Unbonding during
        // ApplyAndReturnValidatorSetUpdates and then Unbonding -> Unbonded during
        // UnbondAllMatureValidatorQueue).
        let validator_updates = self.apply_and_return_validator_setup_dates(ctx)?;

        // unbond all mature validators from the unbonding queue
        self.unbond_all_mature_validators(ctx)?;

        // Remove all mature unbonding delegations from the ubd queue.
        let mature_unbonds = self.dequeue_all_mature_ubd_queue(ctx, Utc::now())?;
        for dv_pair in mature_unbonds {
            let val_addr = dv_pair.val_addr;
            let val_addr_str = val_addr.to_string();
            let del_addr = dv_pair.del_addr;
            let del_addr_str = del_addr.to_string();
            let balances = self.complete_unbonding(ctx, val_addr, del_addr)?;

            ctx.push_event(Event {
                r#type: EVENT_TYPE_COMPLETE_UNBONDING.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.as_bytes().into(),
                        value: serde_json::to_vec(&balances)?.into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: val_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_DELEGATOR.as_bytes().into(),
                        value: del_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                ],
            });
        }
        // Remove all mature redelegations from the red queue.
        let mature_redelegations = self.dequeue_all_mature_redelegation_queue(ctx, Utc::now())?;
        for dvv_triplet in mature_redelegations {
            let val_src_addr = dvv_triplet.val_src_addr;
            let val_src_addr_str = val_src_addr.to_string();
            let val_dst_addr = dvv_triplet.val_dst_addr;
            let val_dst_addr_str = val_dst_addr.to_string();
            let del_addr = dvv_triplet.del_addr;
            let del_addr_str = del_addr.to_string();
            let balances = self.complete_redelegation(ctx, del_addr, val_src_addr, val_dst_addr)?;
            ctx.push_event(Event {
                r#type: EVENT_TYPE_COMPLETE_REDELEGATION.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.as_bytes().into(),
                        value: serde_json::to_vec(&balances)?.into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_DELEGATOR.as_bytes().into(),
                        value: del_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: val_src_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: val_dst_addr_str.as_bytes().to_vec().into(),
                        index: false,
                    },
                ],
            });
        }
        Ok(validator_updates)
    }

    /// ApplyAndReturnValidatorSetUpdates applies and return accumulated updates to the bonded validator set. Also,
    /// * Updates the active valset as keyed by LastValidatorPowerKey.
    /// * Updates the total power as keyed by LastTotalPowerKey.
    /// * Updates validator status' according to updated powers.
    /// * Updates the fee pool bonded vs not-bonded tokens.
    /// * Updates relevant indices.
    /// It gets called once after genesis, another time maybe after genesis transactions,
    /// then once at every EndBlock.
    ///
    /// CONTRACT: Only validators with non-zero power or zero-power that were bonded
    /// at the previous block height or were removed from the validator set entirely
    /// are returned to Tendermint.
    pub fn apply_and_return_validator_setup_dates<
        DB: Database,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
    ) -> anyhow::Result<Vec<ValidatorUpdate>> {
        let params = self.staking_params_keeper.get(&ctx.multi_store())?;
        let max_validators = params.max_validators;
        let power_reduction = self.power_reduction(ctx);
        let mut total_power = 0;
        let mut amt_from_bonded_to_not_bonded: i64 = 0;
        let amt_from_not_bonded_to_bonded: i64 = 0;

        let mut last = self.get_last_validators_by_addr(ctx)?;
        let validators_map = self.get_validators_power_store_vals_map(ctx)?;

        let mut updates = vec![];

        for (_k, val_addr) in validators_map.iter().take(max_validators as usize) {
            // everything that is iterated in this loop is becoming or already a
            // part of the bonded validator set
            let mut validator: Validator =
                self.get_validator(ctx, val_addr.to_string().as_bytes())?;

            if validator.jailed {
                return Err(AppError::Custom(
                    "should never retrieve a jailed validator from the power store".to_string(),
                )
                .into());
            }
            // if we get to a zero-power validator (which we don't bond),
            // there are no more possible bonded validators
            if validator.tokens_to_consensus_power(self.power_reduction(ctx)) == 0 {
                break;
            }

            // apply the appropriate state change if necessary
            match validator.status {
                BondStatus::Unbonded => {
                    self.unbonded_to_bonded(ctx, &mut validator)?;
                    amt_from_bonded_to_not_bonded =
                        amt_from_not_bonded_to_bonded + validator.tokens.amount.parse::<i64>()?;
                }
                BondStatus::Unbonding => {
                    self.unbonding_to_bonded(ctx, &mut validator)?;
                    amt_from_bonded_to_not_bonded =
                        amt_from_not_bonded_to_bonded + validator.tokens.amount.parse::<i64>()?;
                }
                BondStatus::Bonded => {}
            }

            // fetch the old power bytes
            let val_addr_str = val_addr.to_string();
            let old_power_bytes = last.get(&val_addr_str);
            let new_power = validator.consensus_power(power_reduction);
            let new_power_bytes = new_power.to_ne_bytes();
            // update the validator set if power has changed
            if old_power_bytes.is_none()
                || old_power_bytes.map(|v| v.as_slice()) != Some(&new_power_bytes)
            {
                updates.push(validator.abci_validator_update(power_reduction));

                self.set_last_validator_power(
                    ctx,
                    &LastValidatorPower {
                        address: val_addr.clone(),
                        power: new_power,
                    },
                )?;
            }

            last.remove(&val_addr_str);

            total_power += new_power;
        }

        let no_longer_bonded = sort_no_longer_bonded(&last)?;

        for val_addr_bytes in no_longer_bonded {
            let mut validator = self.get_validator(ctx, val_addr_bytes.as_bytes())?;
            self.bonded_to_unbonding(ctx, &mut validator)?;
            amt_from_bonded_to_not_bonded =
                amt_from_not_bonded_to_bonded + validator.tokens.amount.parse::<i64>()?;
            self.delete_last_validator_power(ctx, &validator.operator_address)?;
            updates.push(validator.abci_validator_update_zero());
        }

        // Update the pools based on the recent updates in the validator set:
        // - The tokens from the non-bonded candidates that enter the new validator set need to be transferred
        // to the Bonded pool.
        // - The tokens from the bonded validators that are being kicked out from the validator set
        // need to be transferred to the NotBonded pool.
        // Compare and subtract the respective amounts to only perform one transfer.
        // This is done in order to avoid doing multiple updates inside each iterator/loop.
        match amt_from_bonded_to_not_bonded.cmp(&amt_from_not_bonded_to_bonded) {
            Ordering::Greater => {
                self.not_bonded_tokens_to_bonded(
                    ctx,
                    amt_from_bonded_to_not_bonded - amt_from_not_bonded_to_bonded,
                )?;
            }
            Ordering::Less => {
                self.bonded_tokens_to_not_bonded(
                    ctx,
                    amt_from_bonded_to_not_bonded - amt_from_not_bonded_to_bonded,
                )?;
            }
            Ordering::Equal => {}
        }

        // set total power on lookup index if there are any updates
        if !updates.is_empty() {
            self.set_last_total_power(ctx, Uint256::from_u128(total_power as u128));
        }
        Ok(updates)
    }

    pub fn unbond_all_mature_validators<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> anyhow::Result<()> {
        // TODO: time in ctx
        let block_time = Utc::now();
        let block_height = ctx.height() as i64;

        // unbondingValIterator will contains all validator addresses indexed under
        // the ValidatorQueueKey prefix. Note, the entire index key is composed as
        // ValidatorQueueKey | timeBzLen (8-byte big endian) | timeBz | heightBz (8-byte big endian),
        // so it may be possible that certain validator addresses that are iterated
        // over are not ready to unbond, so an explicit check is required.
        let unbonding_val_iterator: HashMap<Vec<u8>, Vec<String>> =
            self.validator_queue_iterator(ctx, block_time, block_height)?;

        for (k, v) in &unbonding_val_iterator {
            let (time, height) = parse_validator_queue_key(k)?;

            // All addresses for the given key have the same unbonding height and time.
            // We only unbond if the height and time are less than the current height
            // and time.

            if height < block_height && (time <= block_time) {
                for addr in v {
                    let mut validator = self.get_validator(ctx, addr.as_bytes())?;
                    if validator.status != BondStatus::Unbonding {
                        return Err(AppError::Custom(
                            "unexpected validator in unbonding queue; status was not unbonding"
                                .into(),
                        )
                        .into());
                    }
                    self.unbonding_to_unbonded(ctx, &mut validator)?;
                    if validator.delegator_shares.is_zero() {
                        self.remove_validator(
                            ctx,
                            validator.operator_address.to_string().as_bytes(),
                        )?;
                    }
                }
            }

            let store = ctx.kv_store_mut(&self.store_key);
            let mut store = store.prefix_store_mut(VALIDATORS_QUEUE_KEY);
            unbonding_val_iterator.keys().for_each(|k| {
                store.delete(k);
            });
        }
        Ok(())
    }

    pub fn power_reduction<DB: Database, CTX: QueryableContext<DB, SK>>(&self, _ctx: &CTX) -> i64 {
        // TODO: sdk constant in cosmos
        1_000_000
    }

    pub fn not_bonded_tokens_to_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        amount: i64,
    ) -> anyhow::Result<()> {
        let params = self.staking_params_keeper.get(&ctx.multi_store())?;
        let coins = SendCoins::new(vec![Coin {
            denom: params.bond_denom,
            amount: Uint256::from(amount as u64),
        }])?;
        Ok(self
            .bank_keeper
            .send_coins_from_module_to_module::<DB, AK, CTX>(
                ctx,
                NOT_BONDED_POOL_NAME.into(),
                BONDED_POOL_NAME.into(),
                coins,
            )?)
    }

    pub fn bonded_tokens_to_not_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        amount: i64,
    ) -> anyhow::Result<()> {
        let params = self.staking_params_keeper.get(&ctx.multi_store())?;
        let coins = SendCoins::new(vec![Coin {
            denom: params.bond_denom,
            amount: Uint256::from(amount as u64),
        }])?;
        Ok(self
            .bank_keeper
            .send_coins_from_module_to_module::<DB, AK, CTX>(
                ctx,
                BONDED_POOL_NAME.into(),
                NOT_BONDED_POOL_NAME.into(),
                coins,
            )?)
    }

    pub fn bonded_to_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Bonded {
            return Err(AppError::Custom(format!(
                "bad state transition bonded to unbonding, validator: {:?}",
                validator
            ))
            .into());
        }
        self.begin_unbonding_validator(ctx, validator)
    }

    pub fn unbonded_to_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Unbonded {
            return Err(AppError::Custom(format!(
                "bad state transition unbonded to bonded, validator: {:?}",
                validator
            ))
            .into());
        }
        self.bond_validator(ctx, validator)
    }

    pub fn unbonding_to_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Unbonding {
            return Err(AppError::Custom(format!(
                "bad state transition unbonding to bonded, validator: {:?}",
                validator
            ))
            .into());
        }
        self.bond_validator(ctx, validator)
    }

    pub fn unbonding_to_unbonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Unbonding {
            return Err(AppError::Custom(format!(
                "bad state transition unbonding to unbonded, validator: {:?}",
                validator
            ))
            .into());
        }
        self.complete_unbonding_validator(ctx, validator)?;
        Ok(())
    }

    pub fn complete_redelegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_src_addr: ValAddress,
        val_dst_addr: ValAddress,
    ) -> anyhow::Result<Vec<Coin>> {
        let redelegation = self.get_redelegation(ctx, del_addr, val_src_addr, val_dst_addr)?;

        let mut balances = vec![];
        let ctx_time = Utc::now();

        // loop through all the entries and complete mature redelegation entries
        let mut new_redelegations = vec![];
        for entry in &redelegation.entries {
            if entry.is_mature(ctx_time) && !entry.initial_balance.amount.is_zero() {
                balances.push(entry.initial_balance.clone());
            } else {
                new_redelegations.push(entry);
            }
        }

        // set the redelegation or remove it if there are no more entries
        if new_redelegations.is_empty() {
            self.remove_redelegation(ctx, &redelegation);
        } else {
            self.set_redelegation(ctx, &redelegation)?;
        }
        Ok(balances)
    }

    pub fn complete_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
        del_addr: AccAddress,
    ) -> anyhow::Result<Vec<Coin>> {
        let params = self.staking_params_keeper.get(&ctx.multi_store())?;
        let ubd = if let Some(delegation) = self.get_unbonding_delegation(ctx, del_addr, val_addr) {
            delegation
        } else {
            return Err(AppError::Custom("No unbonding delegation".into()).into());
        };
        let bond_denom = params.bond_denom;
        let mut balances = vec![];
        let ctx_time = Utc::now();

        // loop through all the entries and complete unbonding mature entries
        let mut new_ubd = vec![];
        for entry in ubd.entries.iter() {
            if entry.is_mature(ctx_time) {
                // track undelegation only when remaining or truncated shares are non-zero
                if entry.balance.amount.is_zero() {
                    let coin = Coin {
                        denom: bond_denom.clone(),
                        amount: entry.balance.amount,
                    };
                    let amount = SendCoins::new(vec![coin.clone()])?;
                    self.bank_keeper
                        .undelegate_coins_from_module_to_account::<DB, AK, CTX>(
                            ctx,
                            NOT_BONDED_POOL_NAME.to_string(),
                            ubd.delegator_address.clone(),
                            amount,
                        )?;
                    balances.push(coin);
                }
            } else {
                new_ubd.push(entry.clone());
            }
        }

        // set the unbonding delegation or remove it if there are no more entries
        if new_ubd.is_empty() {
            self.remove_unbonding_delegation(ctx, &ubd);
        } else {
            self.set_unbonding_delegation(ctx, &ubd)?;
        }

        Ok(balances)
    }

    pub fn complete_unbonding_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        validator.update_status(BondStatus::Unbonded);
        self.set_validator(ctx, validator)
    }

    pub fn begin_unbonding_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        // delete the validator by power index, as the key will change
        self.delete_validator_by_power_index(ctx, validator);
        // sanity check
        if validator.status != BondStatus::Bonded {
            return Err(AppError::Custom(format!(
                "should not already be unbonded or unbonding, validator: {:?}",
                validator
            ))
            .into());
        }
        validator.update_status(BondStatus::Unbonding);

        // set the unbonding completion time and completion height appropriately
        // TODO: time in ctx
        validator.unbonding_time = Utc::now();
        validator.unbonding_height = ctx.height() as i64;

        // save the now unbonded validator record and power index
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;

        // Adds to unbonding validator queue
        self.insert_unbonding_validator_queue(ctx, validator)?;

        // trigger hook
        self.after_validator_begin_unbonding(ctx, validator)?;
        Ok(())
    }

    pub fn bond_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        // delete the validator by power index, as the key will change
        self.delete_validator_by_power_index(ctx, validator);

        validator.update_status(BondStatus::Bonded);
        // save the now bonded validator record to the two referenced stores
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;

        // delete from queue if present
        self.delete_validator_queue(ctx, validator)?;
        // trigger hook
        self.after_validator_bonded(ctx, validator)?;
        Ok(())
    }

    pub fn validator_queue_iterator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        block_time: chrono::DateTime<Utc>,
        block_height: i64,
    ) -> anyhow::Result<HashMap<Vec<u8>, Vec<String>>> {
        let store = ctx.kv_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATORS_QUEUE_KEY);

        let end = {
            let mut k = get_validator_queue_key(block_time, block_height);
            k.push(0);
            k
        };

        let mut res = HashMap::new();
        for (k, v) in iterator.range(..).take_while(|(k, _)| **k != end) {
            res.insert(k.to_vec(), serde_json::from_slice(&v)?);
        }
        Ok(res)
    }

    pub fn set_unbonding_validators_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        end_time: chrono::DateTime<Utc>,
        end_height: i64,
        addrs: Vec<String>,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(VALIDATORS_QUEUE_KEY);
        let key = get_validator_queue_key(end_time, end_height);
        let value = serde_json::to_vec(&addrs)?;
        store.set(key, value);
        Ok(())
    }

    /// DeleteValidatorQueueTimeSlice deletes all entries in the queue indexed by a
    /// given height and time.
    pub fn delete_validator_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        end_time: chrono::DateTime<Utc>,
        end_height: i64,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(VALIDATORS_QUEUE_KEY);
        store.delete(&get_validator_queue_key(end_time, end_height));
    }

    pub fn delete_validator_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        let addrs = self.get_unbonding_validators(
            ctx,
            validator.unbonding_time,
            validator.unbonding_height,
        )?;
        let val_addr = validator.operator_address.to_string();
        let new_addrs = addrs
            .into_iter()
            .filter(|addr| val_addr != **addr)
            .collect::<Vec<_>>();
        if new_addrs.is_empty() {
            self.delete_validator_queue_time_slice(
                ctx,
                validator.unbonding_time,
                validator.unbonding_height,
            );
        } else {
            self.set_unbonding_validators_queue(
                ctx,
                validator.unbonding_time,
                validator.unbonding_height,
                new_addrs,
            )?;
        }
        Ok(())
    }

    pub fn get_unbonding_validators<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        unbonding_time: chrono::DateTime<Utc>,
        unbonding_height: i64,
    ) -> anyhow::Result<Vec<String>> {
        let store = ctx.kv_store_mut(&self.store_key);
        let store = store.prefix_store(VALIDATORS_QUEUE_KEY);

        if let Some(bz) = store.get(&get_validator_queue_key(unbonding_time, unbonding_height)) {
            let res: Vec<String> = serde_json::from_slice(&bz)?;
            Ok(res)
        } else {
            Ok(vec![])
        }
    }
}

fn get_validator_queue_key(end_time: chrono::DateTime<Utc>, end_height: i64) -> Vec<u8> {
    let height_bz = end_height.to_ne_bytes();
    let time_bz = end_time
        .timestamp_nanos_opt()
        .expect("Unknown time conversion error")
        .to_ne_bytes();

    let mut bz = VALIDATORS_QUEUE_KEY.to_vec();
    bz.extend_from_slice(&(time_bz.len() as u64).to_ne_bytes());
    bz.extend_from_slice(&time_bz);
    bz.extend_from_slice(&height_bz);
    bz
}

fn get_unbonding_delegation_time_key(time: chrono::DateTime<Utc>) -> Vec<u8> {
    time.timestamp_nanos_opt()
        .expect("Unknown time conversion error")
        .to_ne_bytes()
        .to_vec()
}

fn parse_validator_queue_key(key: &Vec<u8>) -> anyhow::Result<(chrono::DateTime<Utc>, i64)> {
    let prefix_len = VALIDATORS_QUEUE_KEY.len();
    if key[..prefix_len] != VALIDATORS_QUEUE_KEY {
        return Err(
            AppError::Custom("Invalid validators queue key. Invalid prefix.".into()).into(),
        );
    }
    let time_len = u64::from_ne_bytes(key[prefix_len..prefix_len + 8].try_into()?);
    let time = chrono::DateTime::from_timestamp_nanos(i64::from_ne_bytes(
        key[prefix_len + 8..prefix_len + 8 + time_len as usize].try_into()?,
    ));
    let height = i64::from_ne_bytes(key[prefix_len + 8 + time_len as usize..].try_into()?);
    Ok((time, height))
}

/// given a map of remaining validators to previous bonded power
/// returns the list of validators to be unbonded, sorted by operator address
fn sort_no_longer_bonded(last: &HashMap<String, Vec<u8>>) -> anyhow::Result<Vec<String>> {
    let mut no_longer_bonded = last.iter().map(|(k, _v)| k.clone()).collect::<Vec<_>>();
    // sorted by address - order doesn't matter
    no_longer_bonded.sort();
    Ok(no_longer_bonded)
}
