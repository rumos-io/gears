use crate::consts::expect::{SERDE_DECODING_DOMAIN_TYPE, TIMESTAMP_NANOS_EXPECT};

pub use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn redelegation<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_src_addr: ValAddress,
        val_dst_addr: ValAddress,
    ) -> anyhow::Result<Redelegation> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(REDELEGATIONS_KEY);
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
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(REDELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_src_address.to_string().as_bytes());
        key.put(delegation.validator_dst_address.to_string().as_bytes());
        delegations_store.set(
            key,
            serde_json::to_vec(&delegation).expect(SERDE_ENCODING_DOMAIN_TYPE),
        );
    }

    pub fn remove_redelegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Redelegation,
    ) -> Option<Vec<u8>> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(REDELEGATIONS_KEY);
        let mut key = delegation.delegator_address.to_string().as_bytes().to_vec();
        key.put(delegation.validator_src_address.to_string().as_bytes());
        key.put(delegation.validator_dst_address.to_string().as_bytes());
        delegations_store.delete(&key)
    }

    pub fn complete_redelegation<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        del_addr: AccAddress,
        val_src_addr: ValAddress,
        val_dst_addr: ValAddress,
    ) -> anyhow::Result<Vec<Coin>> {
        let redelegation = self.redelegation(ctx, del_addr, val_src_addr, val_dst_addr)?;

        let mut balances = vec![];
        let params = self.staking_params_keeper.get(&ctx.multi_store());
        let denom = params.bond_denom;
        let ctx_time = ctx
            .header
            .time
            .as_ref()
            .expect("Expected timestamp in transaction context header.");
        let ctx_time = chrono::DateTime::from_timestamp(ctx_time.seconds, ctx_time.nanos as u32)
            .expect(
                "Invalid timestamp in transaction header. It means that timestamp contains out-of-range number of seconds and/or invalid nanosecond",
            );

        // loop through all the entries and complete mature redelegation entries
        let mut new_redelegations = vec![];
        for entry in &redelegation.entries {
            let coin = Coin {
                denom: denom.clone(),
                amount: entry.initial_balance,
            };
            if entry.is_mature(ctx_time) && !coin.amount.is_zero() {
                balances.push(coin);
            } else {
                new_redelegations.push(entry);
            }
        }

        // set the redelegation or remove it if there are no more entries
        if new_redelegations.is_empty() {
            self.remove_redelegation(ctx, &redelegation);
        } else {
            self.set_redelegation(ctx, &redelegation);
        }
        Ok(balances)
    }

    pub fn insert_redelegation_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        redelegation: &Redelegation,
        completion_time: chrono::DateTime<Utc>,
    ) {
        let mut time_slice = self.redelegation_queue_time_slice(ctx, completion_time);
        let dvv_triplet = DvvTriplet {
            del_addr: redelegation.delegator_address.clone(),
            val_src_addr: redelegation.validator_src_address.clone(),
            val_dst_addr: redelegation.validator_dst_address.clone(),
        };
        if time_slice.is_empty() {
            self.set_redelegation_queue_time_slice(ctx, completion_time, vec![dvv_triplet]);
        } else {
            time_slice.push(dvv_triplet);
            self.set_redelegation_queue_time_slice(ctx, completion_time, time_slice);
        }
    }

    pub fn redelegation_queue_time_slice<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        completion_time: chrono::DateTime<Utc>,
    ) -> Vec<DvvTriplet> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(REDELEGATION_QUEUE_KEY);

        let key = completion_time
            .timestamp_nanos_opt()
            .expect(TIMESTAMP_NANOS_EXPECT)
            .to_ne_bytes();
        if let Some(bytes) = store.get(&key) {
            serde_json::from_slice(&bytes).expect(SERDE_ENCODING_DOMAIN_TYPE)
        } else {
            vec![]
        }
    }

    pub fn set_redelegation_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        completion_time: chrono::DateTime<Utc>,
        redelegations: Vec<DvvTriplet>,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(REDELEGATION_QUEUE_KEY);

        let key = completion_time
            .timestamp_nanos_opt()
            .expect(TIMESTAMP_NANOS_EXPECT)
            .to_ne_bytes();
        let value = serde_json::to_vec(&redelegations).expect(SERDE_ENCODING_DOMAIN_TYPE);
        store.set(key, value);
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
    ) -> Vec<DvvTriplet> {
        let (keys, mature_redelegations) = {
            let storage = ctx.kv_store(&self.store_key);
            let store = storage.prefix_store(REDELEGATION_QUEUE_KEY);

            // TODO: check
            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            let end = {
                let mut k = unbonding_delegation_time_key(time).to_vec();
                k.push(0);
                k
            };
            let mut mature_redelegations = vec![];
            let mut keys = vec![];
            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            for (k, v) in store.range(..).take_while(|(k, _)| **k != end) {
                let time_slice: Vec<DvvTriplet> =
                    serde_json::from_slice(&v).expect(SERDE_DECODING_DOMAIN_TYPE);
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
        mature_redelegations
    }
}
