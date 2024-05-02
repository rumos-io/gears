use crate::{
    Delegation, GenesisState, LastValidatorPower, Pool, Redelegation, StakingParamsKeeper,
    UnbondingDelegation, Validator,
};
use chrono::Utc;
use gears::{
    core::address::{AccAddress, ValAddress},
    crypto::keys::ReadAccAddress,
    error::AppError,
    params::ParamsSubspaceKey,
    store::{
        database::{Database, PrefixDB},
        QueryableKVStore, ReadPrefixStore, StoreKey, TransactionalKVStore, WritePrefixStore,
    },
    types::{
        base::send::SendCoins,
        context::{init::InitContext, QueryableContext, TransactionalContext},
        decimal256::Decimal256,
        uint::Uint256,
    },
    x::keepers::auth::AuthKeeper,
};
use prost::{bytes::BufMut, Message};
use serde::de::Error;

pub trait Codec {}

/// AccountKeeper defines the expected account keeper methods (noalias)
// TODO: AuthKeeper should implements module account stuff
pub trait AccountKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    fn address_codec() -> Box<dyn Codec> {
        todo!()
    }
    // // TODO: should be a sdk account interface
    fn get_account<DB: Database, AK: AuthKeeper<SK>, CTX: QueryableContext<DB, SK>>(
        _ctx: CTX,
        _addr: ValAddress,
    ) -> AK {
        todo!()
    }
    // only used for simulation
    fn get_module_address(_name: String) -> ValAddress {
        todo!()
    }
    fn get_module_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        _ctx: &CTX,
        _module_name: String,
    ) -> Self {
        todo!()
    }
    fn set_module_account<DB: Database, AK: AuthKeeper<SK>, CTX: QueryableContext<DB, SK>>(
        _context: &CTX,
        _acc: AK,
    ) {
        todo!()
    }
}

pub trait BankKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    fn delegate_coins<DB: Database, AK: AuthKeeper<SK>, CTX: TransactionalContext<DB, SK>>(
        _ctx: &mut CTX,
        _addr: AccAddress,
        _amt: SendCoins,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, AppError> {
        todo!()
    }
    fn undelegate_coins<DB: Database, AK: AuthKeeper<SK>, CTX: TransactionalContext<DB, SK>>(
        _ctx: &mut CTX,
        _addr: AccAddress,
        _amt: SendCoins,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, AppError> {
        todo!()
    }
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
        // ctx = ctx.WithBlockHeight(1 - sdk.ValidatorUpdateDelay)

        self.set_pool(ctx, genesis.pool)?;
        self.set_last_total_power(ctx, genesis.last_total_power);
        self.staking_params_keeper
            .set(&mut ctx.multi_store_mut(), genesis.params)?;

        genesis.validators.iter().for_each(|_v| todo!());

        for validator in genesis.validators {
            self.set_validator(ctx, &validator)?;
            self.set_validator_by_cons_addr(ctx, &validator)?;
            // TODO
            // self.set_validator_by_power_index(ctx, validator)?;
            if !genesis.exported {
                self.after_validator_created(ctx, &validator)?;
            }

            // TODO
            // if validator.status == BondStatus::Unbonding {
            //     self.insert_validator_queue(ctx, &validator)
            // }
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
    pub fn get_last_total_power<DB: Database, CTX: QueryableContext<PrefixDB<DB>, SK>>(
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

    pub fn get_validator<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
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
            validator.operator_address.to_string().encode_to_vec(),
            serde_json::to_vec(&validator)?,
        );
        Ok(())
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

    // pub fn set_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
    //     &self,
    //     ctx: &mut CTX,
    //     validator: &Validator,
    // ) -> anyhow::Result<()> {
    //     if !validator.jailed {
    //         let store = ctx.kv_store_mut(&self.store_key);
    //         let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
    //         todo!()
    //     }
    //     Ok(())
    // }

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

    // TODO: the other key
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

    /// Insert an unbonding delegation to the appropriate timeslice in the unbonding queue
    pub fn insert_ubd_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &UnbondingDelegation,
        time: chrono::DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let _store = ctx.kv_store_mut(&self.store_key);
        let _time_slice = self.get_ubd_queue_time_slice(ctx, time);
        let _dv_pair = (
            delegation.delegator_address.clone(),
            delegation.validator_address.clone(),
        );

        // TODO
        Ok(())
        // if let Some(time_slice) = time_slice {
        //     let time_slice = time_slice.append(dv_pair);
        // //     k.SetUBDQueueTimeSlice(ctx, completionTime, timeSlice)
        // } else {
        //     // self.set_ubd_queue_time_slice(ctx, completionTime, dv_pair);
        // }
    }

    pub fn get_ubd_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: chrono::DateTime<Utc>,
    ) -> Option<(AccAddress, ValAddress)> {
        let store = ctx.kv_store_mut(&self.store_key);
        if let Some(_bz) = store.get(time.to_string().as_bytes()) {
            // TODO
            None
        } else {
            None
        }
    }

    // TODO: the other key
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

    // TODO: the other key
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
}
