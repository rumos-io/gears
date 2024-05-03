pub use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
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
}
