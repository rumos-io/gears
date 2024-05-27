pub use super::*;
use crate::consts::error::{SERDE_ENCODING_DOMAIN_TYPE, TIMESTAMP_NANOS_EXPECT};
use gears::{store::database::ext::DATABASE_CORRUPTION_MSG, tendermint::types::time::Timestamp};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn unbonding_delegation<DB: Database, CTX: QueryableContext<DB, SK>>(
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
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(DELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_address.to_string().as_bytes());
        delegations_store.set(
            key,
            serde_json::to_vec(&delegation).expect(SERDE_ENCODING_DOMAIN_TYPE),
        );
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
    pub fn dequeue_all_mature_ubd_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: chrono::DateTime<Utc>,
    ) -> Vec<DvPair> {
        let (keys, mature_unbonds) = {
            let storage = ctx.kv_store(&self.store_key);
            let store = storage.prefix_store(UNBONDING_QUEUE_KEY);
            let end = unbonding_delegation_time_key(time).to_vec();
            let mut mature_unbonds = vec![];
            let mut keys = vec![];
            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            let mut previous_was_end = false;
            for (k, v) in store.range(..).take_while(|(k, _)| {
                let is_not_end = **k != end;
                let res = is_not_end && !previous_was_end;
                previous_was_end = !is_not_end;
                res
            }) {
                let time_slice: Vec<DvPair> =
                    serde_json::from_slice(&v).expect(DATABASE_CORRUPTION_MSG);
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
        mature_unbonds
    }

    /// Insert an unbonding delegation to the appropriate timeslice in the unbonding queue
    pub fn insert_ubd_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
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
    ) {
        let mut addrs =
            self.unbonding_validators(ctx, &validator.unbonding_time, validator.unbonding_height);
        addrs.push(validator.operator_address.to_string());
        self.set_unbonding_validators_queue(
            ctx,
            validator.unbonding_time.clone(),
            validator.unbonding_height,
            addrs,
        );
    }

    pub fn ubd_queue_time_slice<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: &Timestamp,
    ) -> Option<Vec<DvPair>> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(UBD_QUEUE_KEY);
        // TODO: consider to move the DataTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let time = chrono::DateTime::from_timestamp(time.seconds, time.nanos as u32).unwrap();
        if let Some(bz) = store.get(time.to_string().as_bytes()) {
            serde_json::from_slice(&bz).unwrap_or_default()
        } else {
            None
        }
    }

    pub fn set_ubd_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        time: Timestamp,
        time_slice: Vec<DvPair>,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(UBD_QUEUE_KEY);
        // TODO: consider to move the DataTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let time = chrono::DateTime::from_timestamp(time.seconds, time.nanos as u32).unwrap();
        let key = time.to_string().as_bytes().to_vec();
        store.set(
            key,
            serde_json::to_vec(&time_slice).expect(SERDE_ENCODING_DOMAIN_TYPE),
        );
    }

    pub fn set_last_validator_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &LastValidatorPower,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(LAST_VALIDATOR_POWER_KEY);
        let key = validator.address.to_string().as_bytes().to_vec();
        delegations_store.set(
            key,
            serde_json::to_vec(&validator).expect(SERDE_ENCODING_DOMAIN_TYPE),
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

    pub fn unbond_all_mature_validators<DB: Database>(&self, ctx: &mut BlockContext<'_, DB, SK>) {
        // TODO: make better api for timestamps in Gears
        let block_time = ctx.get_time().unwrap();
        // TODO: consider to move the DataTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let block_time =
            chrono::DateTime::from_timestamp(block_time.seconds, block_time.nanos as u32).unwrap();

        let block_height = ctx.height() as i64;

        // unbondingValIterator will contains all validator addresses indexed under
        // the ValidatorQueueKey prefix. Note, the entire index key is composed as
        // ValidatorQueueKey | timeBzLen (8-byte big endian) | timeBz | heightBz (8-byte big endian),
        // so it may be possible that certain validator addresses that are iterated
        // over are not ready to unbond, so an explicit check is required.
        let unbonding_val_map: HashMap<Vec<u8>, Vec<String>> =
            self.validator_queue_map(ctx, block_time, block_height);

        for (k, v) in &unbonding_val_map {
            let (time, height) =
                parse_validator_queue_key(k).expect("failed to parse unbonding key");

            // All addresses for the given key have the same unbonding height and time.
            // We only unbond if the height and time are less than the current height
            // and time.

            if height < block_height && (time <= block_time) {
                for addr in v {
                    let val_addr = ValAddress::from_bech32(addr)
                        .expect("Failed to parse stored ValAddress in validators queue. Validators queue map contains vector of string addresses that could be a valid ValAddress representation.");
                    let mut validator = self
                        .validator(ctx, &val_addr)
                        .expect("validator in the unbonding queue was not found");

                    assert_eq!(
                        validator.status,
                        BondStatus::Unbonding,
                        "unexpected validator in unbonding queue; status was not unbonding"
                    );

                    self.unbonding_to_unbonded(ctx, &mut validator);
                    if validator.delegator_shares.is_zero() {
                        self.remove_validator(
                            ctx,
                            validator.operator_address.to_string().as_bytes(),
                        );
                    }
                }
            }

            let store = ctx.kv_store_mut(&self.store_key);
            let mut store = store.prefix_store_mut(VALIDATORS_QUEUE_KEY);
            unbonding_val_map.keys().for_each(|k| {
                store.delete(k);
            });
        }
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
        self.bond_validator(ctx, validator);
        Ok(())
    }

    pub fn unbonding_to_unbonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) {
        assert_eq!(
            validator.status,
            BondStatus::Unbonding,
            "bad state transition unbonding to unbonded, validator: {}",
            validator.operator_address
        );
        self.complete_unbonding_validator(ctx, validator);
    }

    pub fn complete_unbonding<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        val_addr: ValAddress,
        del_addr: AccAddress,
    ) -> anyhow::Result<Vec<Coin>> {
        let params = self.staking_params_keeper.get(&ctx.multi_store());
        let ubd = if let Some(delegation) = self.unbonding_delegation(ctx, del_addr, val_addr) {
            delegation
        } else {
            return Err(AppError::Custom("No unbonding delegation".into()).into());
        };
        let bond_denom = params.bond_denom;
        let mut balances = vec![];
        // TODO: make better api for timestamps in Gears
        let ctx_time = ctx.get_time().unwrap();
        // TODO: consider to move the DataTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let ctx_time =
            chrono::DateTime::from_timestamp(ctx_time.seconds, ctx_time.nanos as u32).unwrap();

        // loop through all the entries and complete unbonding mature entries
        let mut new_ubd = vec![];
        for entry in ubd.entries.iter() {
            if entry.is_mature(ctx_time) {
                // track undelegation only when remaining or truncated shares are non-zero
                let amount = entry.balance;
                if amount.is_zero() {
                    let coin = Coin {
                        denom: bond_denom.clone(),
                        amount,
                    };
                    let amount = SendCoins::new(vec![coin.clone()])?;
                    self.bank_keeper
                        .undelegate_coins_from_module_to_account::<DB, AK, BlockContext<'_, DB, SK>>(
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
            self.set_unbonding_delegation(ctx, &ubd);
        }

        Ok(balances)
    }

    pub fn complete_unbonding_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) {
        validator.update_status(BondStatus::Unbonded);
        self.set_validator(ctx, validator);
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
                "should not already be unbonded or unbonding, validator: {}",
                validator.operator_address
            ))
            .into());
        }
        validator.update_status(BondStatus::Unbonding);

        // set the unbonding completion time and completion height appropriately
        // TODO: make better api for timestamps in Gears
        validator.unbonding_time = ctx.get_time().unwrap();
        validator.unbonding_height = ctx.height() as i64;

        // save the now unbonded validator record and power index
        self.set_validator(ctx, validator);
        self.set_validator_by_power_index(ctx, validator);

        // Adds to unbonding validator queue
        self.insert_unbonding_validator_queue(ctx, validator);

        // trigger hook
        self.after_validator_begin_unbonding(ctx, validator)?;
        Ok(())
    }

    pub fn set_unbonding_validators_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        end_time: Timestamp,
        end_height: i64,
        addrs: Vec<String>,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(VALIDATORS_QUEUE_KEY);
        // TODO: consider to move the DataTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let end_time =
            chrono::DateTime::from_timestamp(end_time.seconds, end_time.nanos as u32).unwrap();
        let key = validator_queue_key(end_time, end_height);
        let value = serde_json::to_vec(&addrs).expect(SERDE_ENCODING_DOMAIN_TYPE);
        store.set(key, value);
    }

    /// DeleteValidatorQueueTimeSlice deletes all entries in the queue indexed by a
    /// given height and time.
    pub fn delete_validator_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        end_time: Timestamp,
        end_height: i64,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(VALIDATORS_QUEUE_KEY);
        // TODO: consider to move the DataTime type and work with timestamps into Gears
        // The timestamp is provided by context and conversion won't fail.
        let end_time =
            chrono::DateTime::from_timestamp(end_time.seconds, end_time.nanos as u32).unwrap();
        store.delete(&validator_queue_key(end_time, end_height));
    }

    pub fn unbonding_validators<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        unbonding_time: &Timestamp,
        unbonding_height: i64,
    ) -> Vec<String> {
        let store = ctx.kv_store_mut(&self.store_key);
        let store = store.prefix_store(VALIDATORS_QUEUE_KEY);

        if let Some(bz) = store.get(&validator_queue_key(
            // TODO: consider to move the DataTime type and work with timestamps into Gears
            // The timestamp is provided by context and conversion won't fail.
            chrono::DateTime::from_timestamp(unbonding_time.seconds, unbonding_time.nanos as u32)
                .unwrap(),
            unbonding_height,
        )) {
            let res: Vec<String> = serde_json::from_slice(&bz).expect(DATABASE_CORRUPTION_MSG);
            res
        } else {
            vec![]
        }
    }
}

pub(super) fn unbonding_delegation_time_key(time: chrono::DateTime<Utc>) -> [u8; 8] {
    time.timestamp_nanos_opt()
        .expect(TIMESTAMP_NANOS_EXPECT)
        .to_ne_bytes()
}
