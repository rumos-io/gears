use std::ops::Bound;

use super::*;
use crate::{
    consts::error::SERDE_ENCODING_DOMAIN_TYPE, parse_validator_queue_key,
    unbonding_delegation_time_key, validator_queue_key,
};
use gears::{
    context::{InfallibleContext, InfallibleContextMut},
    error::IBC_ENCODE_UNWRAP,
    store::database::ext::UnwrapCorrupt,
    tendermint::types::{proto::Protobuf, time::Timestamp},
};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    pub fn unbonding_delegation<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    ) -> Option<UnbondingDelegation> {
        let store = InfallibleContext::infallible_store(ctx, &self.store_key);
        let delegations_store = store.prefix_store(UNBONDING_DELEGATION_KEY);
        let mut key = Vec::from(del_addr);
        key.extend_from_slice(&Vec::from(val_addr));
        if let Some(bytes) = delegations_store.get(&key) {
            if let Ok(delegation) = serde_json::from_slice(&bytes) {
                return Some(delegation);
            }
        }
        None
    }

    pub fn set_unbonding_delegation<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &UnbondingDelegation,
    ) {
        let store = InfallibleContextMut::infallible_store_mut(ctx, &self.store_key);
        let mut delegations_store = store.prefix_store_mut(UNBONDING_DELEGATION_KEY);
        let mut key = Vec::from(delegation.delegator_address.clone());
        key.extend_from_slice(&Vec::from(delegation.validator_address.clone()));
        delegations_store.set(
            key,
            serde_json::to_vec(&delegation).expect(SERDE_ENCODING_DOMAIN_TYPE),
        );
    }

    pub fn remove_unbonding_delegation<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &UnbondingDelegation,
    ) -> Option<Vec<u8>> {
        let store = InfallibleContextMut::infallible_store_mut(ctx, &self.store_key);
        let mut delegations_store = store.prefix_store_mut(UNBONDING_DELEGATION_KEY);
        let mut key = Vec::from(delegation.delegator_address.clone());
        key.extend_from_slice(&Vec::from(delegation.validator_address.clone()));
        delegations_store.delete(&key)
    }

    /// Returns a concatenated list of all the timeslices inclusively previous to
    /// currTime, and deletes the timeslices from the queue
    pub fn dequeue_all_mature_ubd_queue<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: &Timestamp,
    ) -> Vec<DvPair> {
        let (keys, mature_unbonds) = {
            let storage = InfallibleContext::infallible_store(ctx, &self.store_key);
            let store = storage.prefix_store(UNBONDING_QUEUE_KEY);
            let end = unbonding_delegation_time_key(time).to_vec();
            let mut mature_unbonds = vec![];
            let mut keys = vec![];
            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            let mut previous_was_end = false;
            for (k, v) in store.into_range(..).take_while(|(k, _)| {
                let is_not_end = **k != end;
                let res = is_not_end && !previous_was_end;
                previous_was_end = !is_not_end;
                res
            }) {
                let time_slice: Vec<DvPair> = serde_json::from_slice(&v).unwrap_or_corrupt();
                mature_unbonds.extend(time_slice);
                keys.push(k.to_vec());
            }
            (keys, mature_unbonds)
        };
        let storage = InfallibleContextMut::infallible_store_mut(ctx, &self.store_key);
        let mut store = storage.prefix_store_mut(UNBONDING_QUEUE_KEY);
        keys.iter().for_each(|k| {
            store.delete(k);
        });
        mature_unbonds
    }

    /// Insert an unbonding delegation to the appropriate timeslice in the unbonding queue
    pub fn insert_ubd_queue<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &UnbondingDelegation,
        time: Timestamp,
    ) {
        let time_slice = self.ubd_queue_time_slice(ctx, &time);
        let dv_pair = DvPair::new(
            delegation.validator_address.clone(),
            delegation.delegator_address.clone(),
        );

        if let Some(mut time_slice) = time_slice {
            time_slice.push(dv_pair);
            self.set_ubd_queue_time_slice(ctx, time, time_slice);
        } else {
            self.set_ubd_queue_time_slice(ctx, time, vec![dv_pair]);
        }
    }

    pub fn insert_unbonding_validator_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<(), GasStoreErrors> {
        let mut addrs =
            self.unbonding_validators(ctx, &validator.unbonding_time, validator.unbonding_height)?;
        addrs.push(validator.operator_address.clone());
        self.set_unbonding_validators_queue(
            ctx,
            validator.unbonding_time.clone(),
            validator.unbonding_height,
            addrs,
        )?;

        Ok(())
    }

    pub fn ubd_queue_time_slice<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: &Timestamp,
    ) -> Option<Vec<DvPair>> {
        let store = InfallibleContext::infallible_store(ctx, &self.store_key);
        let store = store.prefix_store(UNBONDING_QUEUE_KEY);
        if let Some(bz) = store.get(&time.encode_vec().expect(IBC_ENCODE_UNWRAP)) {
            serde_json::from_slice(&bz).unwrap_or_default()
        } else {
            None
        }
    }

    pub fn set_ubd_queue_time_slice<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: Timestamp,
        time_slice: Vec<DvPair>,
    ) {
        let store = InfallibleContextMut::infallible_store_mut(ctx, &self.store_key);
        let mut store = store.prefix_store_mut(UNBONDING_QUEUE_KEY);
        let key = time.encode_vec().expect(IBC_ENCODE_UNWRAP);
        store.set(
            key,
            serde_json::to_vec(&time_slice).expect(SERDE_ENCODING_DOMAIN_TYPE),
        );
    }

    pub fn after_validator_begin_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        if let Some(ref hooks) = self.hooks_keeper {
            hooks.after_validator_begin_unbonding(
                ctx,
                validator.cons_addr(),
                validator.operator_address.clone(),
            );
        }
        Ok(())
    }

    pub fn unbond_all_mature_validators<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
    ) -> Result<(), GasStoreErrors> {
        let block_time = ctx.get_time();
        let block_height = ctx.height() as u64;

        // unbonding_validators_map will contains all validator addresses indexed under
        // the ValidatorQueueKey prefix. Note, the entire index key is composed as
        // ValidatorQueueKey | timeBzLen (8-byte big endian) | timeBz | heightBz (8-byte big endian),
        // so it may be possible that certain validator addresses that are iterated
        // over are not ready to unbond, so an explicit check is required.
        let unbonding_val_map: HashMap<Vec<u8>, Vec<ValAddress>> =
            self.unbonding_validator_queue_map(ctx, &block_time, block_height);
        // TODO: in context of solving issues with shared and mutable references it is need to
        // create owned collection. It's less performant even if we update iterator to infallible
        // version.
        // The sdk allows to iterate over a store without resolving the
        // possible issues with lifetimes.
        // let unbonding_val_map: HashMap<Vec<u8>, Vec<ValAddress>> = self
        //     .unbonding_validator_queue_iter(ctx, &block_time, block_height)
        //     .map(|r| {
        //         let (k, v) = r.unwrap_gas();
        //         (k.to_vec(), v)
        //     })
        //     .collect();

        // TODO: consider to move the DateTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let block_time =
            chrono::DateTime::from_timestamp(block_time.seconds, block_time.nanos as u32).unwrap();

        for (k, v) in &unbonding_val_map {
            let (time, height) =
                parse_validator_queue_key(k).expect("failed to parse unbonding key");

            // All addresses for the given key have the same unbonding height and time.
            // We only unbond if the height and time are less than the current height
            // and time.

            if height < block_height && (time <= block_time) {
                for val_addr in v {
                    let mut validator = self
                        .validator(ctx, val_addr)?
                        .expect("validator in the unbonding queue was not found");

                    assert_eq!(
                        validator.status,
                        BondStatus::Unbonding,
                        "unexpected validator in unbonding queue; status was not unbonding"
                    );

                    self.unbonding_to_unbonded(ctx, &mut validator)
                        .expect(CTX_NO_GAS_UNWRAP);
                    if validator.delegator_shares.is_zero() {
                        self.remove_validator(ctx, &validator)?;
                    }
                }
            }

            let store = ctx.kv_store_mut(&self.store_key);
            let mut store = store.prefix_store_mut(VALIDATOR_QUEUE_KEY);
            unbonding_val_map.keys().for_each(|k| {
                store.delete(k);
            });
        }

        Ok(())
    }

    pub fn unbonding_to_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Unbonding {
            return Err(AppError::Custom(format!(
                "bad state transition unbonding to bonded, validator: {}",
                validator.operator_address
            ))
            .into());
        }
        self.bond_validator(ctx, validator)?;
        Ok(())
    }

    pub fn unbonding_to_unbonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), GasStoreErrors> {
        assert_eq!(
            validator.status,
            BondStatus::Unbonding,
            "bad state transition unbonding to unbonded, validator: {}",
            validator.operator_address
        );
        self.complete_unbonding_validator(ctx, validator)
    }

    pub fn complete_unbonding<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        val_addr: ValAddress,
        del_addr: AccAddress,
    ) -> anyhow::Result<Vec<Coin>> {
        let params = self.staking_params_keeper.get(ctx);
        let ubd = if let Some(delegation) = self.unbonding_delegation(ctx, del_addr, val_addr) {
            delegation
        } else {
            return Err(AppError::Custom("No unbonding delegation".into()).into());
        };
        let bond_denom = params.bond_denom;
        let mut balances = vec![];
        let ctx_time = ctx.get_time();

        // loop through all the entries and complete unbonding mature entries
        let mut new_ubd = vec![];
        for entry in ubd.entries.iter() {
            if entry.is_mature(&ctx_time) {
                // track undelegation only when remaining or truncated shares are non-zero
                let amount = entry.balance;
                if amount.is_zero() {
                    let coin = Coin {
                        denom: bond_denom.clone(),
                        amount,
                    };
                    let amount = SendCoins::new(vec![coin.clone()])?;
                    self.bank_keeper
                        .undelegate_coins_from_module_to_account::<DB, BlockContext<'_, DB, SK>>(
                            ctx,
                            &self.not_bonded_module,
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
            self.set_unbonding_delegation(ctx, &ubd);
        }

        Ok(balances)
    }

    pub fn complete_unbonding_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), GasStoreErrors> {
        validator.update_status(BondStatus::Unbonded);
        self.set_validator(ctx, validator)
    }

    pub fn begin_unbonding_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        // delete the validator by power index, as the key will change
        self.delete_validator_by_power_index(ctx, validator)?;
        // sanity check
        if validator.status != BondStatus::Bonded {
            return Err(AppError::Custom(format!(
                "should not already be unbonded or unbonding, validator: {}",
                validator.operator_address
            ))
            .into());
        }
        validator.update_status(BondStatus::Unbonding);

        // set the unbonding completion time and completion height appropriately
        validator.unbonding_time = ctx.get_time();
        validator.unbonding_height = ctx.height() as u64;

        // save the now unbonded validator record and power index
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;

        // Adds to unbonding validator queue
        self.insert_unbonding_validator_queue(ctx, validator)?;

        // trigger hook
        self.after_validator_begin_unbonding(ctx, validator)?;
        Ok(())
    }

    pub fn unbonding_validators<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        unbonding_time: &Timestamp,
        unbonding_height: u64,
    ) -> Result<Vec<ValAddress>, GasStoreErrors> {
        let store = TransactionalContext::kv_store_mut(ctx, &self.store_key);
        let store = store.prefix_store(VALIDATOR_QUEUE_KEY);

        if let Some(bz) = store.get(&validator_queue_key(unbonding_time, unbonding_height))? {
            let res: Vec<ValAddress> = serde_json::from_slice(&bz).unwrap_or_corrupt();
            Ok(res)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn set_unbonding_validators_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        end_time: Timestamp,
        end_height: u64,
        addrs: Vec<ValAddress>,
    ) -> Result<(), GasStoreErrors> {
        let store = TransactionalContext::kv_store_mut(ctx, &self.store_key);
        let mut store = store.prefix_store_mut(VALIDATOR_QUEUE_KEY);
        let key = validator_queue_key(&end_time, end_height);
        let value = serde_json::to_vec(&addrs).expect(SERDE_ENCODING_DOMAIN_TYPE);
        store.set(key, value)?;
        Ok(())
    }

    /// DeleteValidatorQueueTimeSlice deletes all entries in the queue indexed by a
    /// given height and time.
    pub fn delete_validator_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        end_time: Timestamp,
        end_height: u64,
    ) -> Result<(), GasStoreErrors> {
        let store = TransactionalContext::kv_store_mut(ctx, &self.store_key);
        let mut store = store.prefix_store_mut(VALIDATOR_QUEUE_KEY);
        store.delete(&validator_queue_key(&end_time, end_height))?;
        Ok(())
    }

    pub fn unbonding_validator_queue_map<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
        block_time: &Timestamp,
        block_height: u64,
    ) -> HashMap<Vec<u8>, Vec<ValAddress>> {
        let store = ctx.infallible_store(&self.store_key);
        let start = VALIDATOR_QUEUE_KEY.to_vec();
        let mut end = validator_queue_key(block_time, block_height);
        end.push(0);
        let mut res = HashMap::new();
        for (k, v) in store.into_range((
            Bound::Included(start.clone()),
            Bound::Excluded([start, end].concat()),
        )) {
            res.insert(k.to_vec(), serde_json::from_slice(&v).unwrap_or_corrupt());
        }
        res
    }

    pub fn unbonding_validator_queue_iter<'a, DB: Database, CTX: InfallibleContext<DB, SK>>(
        &'a self,
        ctx: &'a CTX,
        block_time: &Timestamp,
        block_height: u64,
    ) -> UnbondingValidatorsIterator<'a, DB> {
        let store = ctx.kv_store(&self.store_key);
        let start = VALIDATOR_QUEUE_KEY.to_vec();
        let end = validator_queue_key(block_time, block_height);
        UnbondingValidatorsIterator::new(store, start, end)
    }

    pub fn delete_unbonding_validators_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), GasStoreErrors> {
        let addrs =
            self.unbonding_validators(ctx, &validator.unbonding_time, validator.unbonding_height);
        let val_addr = &validator.operator_address;
        let new_addrs = addrs?
            .into_iter()
            .filter(|addr| val_addr != addr)
            .collect::<Vec<_>>();
        if new_addrs.is_empty() {
            self.delete_validator_queue_time_slice(
                ctx,
                validator.unbonding_time.clone(),
                validator.unbonding_height,
            )?;
        } else {
            self.set_unbonding_validators_queue(
                ctx,
                validator.unbonding_time.clone(),
                validator.unbonding_height,
                new_addrs,
            )?;
        }

        Ok(())
    }
}
