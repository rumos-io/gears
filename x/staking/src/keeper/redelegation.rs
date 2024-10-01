use super::*;
use crate::types::keys;
use crate::{
    redelegation_time_key, redelegations_by_delegator_to_validator_destination_index_key,
    DvvTriplets, RedelegationEntry,
};
use gears::context::{InfallibleContext, InfallibleContextMut};
use gears::core::Protobuf;
use gears::extensions::corruption::UnwrapCorrupt;
use prost::bytes::Bytes;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: StakingBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    /// begin unbonding / redelegation; create a redelegation record
    pub fn begin_redelegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_src_addr: &ValAddress,
        val_dst_addr: &ValAddress,
        shares: Decimal256,
    ) -> anyhow::Result<Timestamp> {
        if val_src_addr == val_dst_addr {
            return Err(anyhow::anyhow!("self redelegation".to_string()));
        }

        let mut dst_validator = if let Some(validator) = self.validator(ctx, val_dst_addr)? {
            validator
        } else {
            return Err(anyhow::anyhow!("bad redelegation dst: {}", val_dst_addr));
        };

        let src_validator = if let Some(validator) = self.validator(ctx, val_src_addr)? {
            validator
        } else {
            return Err(anyhow::anyhow!("bad redelegation src: {}", val_dst_addr));
        };

        // check if this is a transitive redelegation
        if self.has_receiving_redelegation(ctx, del_addr, val_src_addr)? {
            return Err(anyhow::anyhow!("transitive redelegation"));
        }

        if self.has_max_redelegation_entries(ctx, del_addr, val_src_addr, val_dst_addr)? {
            return Err(anyhow::anyhow!("max redelegation entries"));
        }

        let return_amount = self.unbond(ctx, del_addr, val_src_addr, shares)?;

        if return_amount.is_zero() {
            return Err(anyhow::anyhow!("tiny redelegation amount"));
        }

        let shares_created = self.delegate(
            ctx,
            del_addr,
            return_amount,
            src_validator.status,
            &mut dst_validator,
            false,
        )?;

        // create the unbonding delegation
        let (completion_time, height, complete_now) = self.begin_info(ctx, val_src_addr)?;
        if complete_now {
            // no need to create the redelegation object
            return Ok(completion_time);
        }

        let redelegation = self.set_redelegation_entry(
            ctx,
            DvvTriplet {
                del_addr: del_addr.clone(),
                val_src_addr: val_src_addr.clone(),
                val_dst_addr: val_dst_addr.clone(),
            },
            height,
            completion_time,
            return_amount,
            shares_created,
        )?;

        self.insert_redelegation_queue(ctx, &redelegation, completion_time)?;
        Ok(completion_time)
    }

    pub fn has_receiving_redelegation<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_src_addr: &ValAddress,
    ) -> Result<bool, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);

        let mut prefix = REDELEGATION_BY_VAL_DST_INDEX_KEY.to_vec();
        let postfix =
            redelegations_by_delegator_to_validator_destination_index_key(val_src_addr, del_addr);
        prefix.extend_from_slice(&postfix);

        Ok(store.prefix_store(prefix).into_range(..).next().is_some())
    }

    pub fn has_max_redelegation_entries<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_src_addr: &ValAddress,
        val_dst_addr: &ValAddress,
    ) -> Result<bool, GasStoreErrors> {
        let params = self.staking_params_keeper.try_get(ctx)?;

        if let Some(redelegation) = self.redelegation(ctx, del_addr, val_src_addr, val_dst_addr)? {
            Ok(redelegation.entries.len() >= params.max_entries() as usize)
        } else {
            Ok(false)
        }
    }

    /// set_redelegation_entry adds an entry to the unbonding delegation at
    /// the given addresses. It creates the unbonding delegation if it does not exist
    pub fn set_redelegation_entry<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        DvvTriplet {
            del_addr,
            val_src_addr,
            val_dst_addr,
        }: DvvTriplet,
        creation_height: u32,
        min_time: Timestamp,
        balance: Uint256,
        shares_dst: Decimal256,
    ) -> Result<Redelegation, GasStoreErrors> {
        let entry = RedelegationEntry {
            creation_height,
            completion_time: min_time,
            initial_balance: balance,
            share_dst: shares_dst,
        };
        let redelegation = if let Some(mut redelegation) =
            self.redelegation(ctx, &del_addr, &val_src_addr, &val_dst_addr)?
        {
            redelegation.add_entry(entry);
            redelegation
        } else {
            Redelegation {
                delegator_address: del_addr,
                validator_src_address: val_src_addr,
                validator_dst_address: val_dst_addr,
                entries: vec![entry],
            }
        };

        self.set_redelegation(ctx, &redelegation)?;
        Ok(redelegation)
    }

    pub fn redelegation<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        del_addr: &AccAddress,
        val_src_addr: &ValAddress,
        val_dst_addr: &ValAddress,
    ) -> Result<Option<Redelegation>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);
        let key = keys::redelegation_key(&del_addr, &val_src_addr, &val_dst_addr);
        Ok(store
            .get(&key)?
            .map(|bytes| Redelegation::decode::<Bytes>(bytes.into()).unwrap_or_corrupt()))
    }

    pub fn set_redelegation<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Redelegation,
    ) -> Result<(), GasStoreErrors> {
        let mut store = ctx.kv_store_mut(&self.store_key);
        store.set(
            keys::redelegation_key(
                &delegation.delegator_address,
                &delegation.validator_src_address,
                &delegation.validator_dst_address,
            ),
            delegation.encode_vec(),
        )?;

        store.set(
            keys::redelegation_by_val_src_index_key(
                &delegation.delegator_address,
                &delegation.validator_src_address,
                &delegation.validator_dst_address,
            ),
            vec![],
        )?;

        store.set(
            keys::redelegation_by_val_dst_index_key(
                &delegation.delegator_address,
                &delegation.validator_src_address,
                &delegation.validator_dst_address,
            ),
            vec![],
        )
    }

    pub fn remove_redelegation<DB: Database, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        delegation: &Redelegation,
    ) -> Option<Vec<u8>> {
        let mut store = InfallibleContextMut::infallible_store_mut(ctx, &self.store_key);
        store.delete(&keys::redelegation_key(
            &delegation.delegator_address,
            &delegation.validator_src_address,
            &delegation.validator_dst_address,
        ));

        store.delete(&keys::redelegation_by_val_src_index_key(
            &delegation.delegator_address,
            &delegation.validator_src_address,
            &delegation.validator_dst_address,
        ));

        store.delete(&keys::redelegation_by_val_dst_index_key(
            &delegation.delegator_address,
            &delegation.validator_src_address,
            &delegation.validator_dst_address,
        ))
    }

    pub fn complete_redelegation<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        del_addr: AccAddress,
        val_src_addr: ValAddress,
        val_dst_addr: ValAddress,
    ) -> anyhow::Result<Vec<UnsignedCoin>> {
        let redelegation = self
            .redelegation(ctx, &del_addr, &val_src_addr, &val_dst_addr)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .ok_or(anyhow::anyhow!("no redelegation found"))?;

        let mut balances = vec![];
        let params = self.staking_params_keeper.get(ctx);
        let denom = params.bond_denom();
        let ctx_time = ctx.header.time;

        // loop through all the entries and complete mature redelegation entries
        let mut new_redelegations = vec![];
        for entry in &redelegation.entries {
            let coin = UnsignedCoin {
                denom: denom.clone(),
                amount: entry.initial_balance,
            };
            if entry.is_mature(&ctx_time) && !coin.amount.is_zero() {
                balances.push(coin);
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

    pub fn insert_redelegation_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        redelegation: &Redelegation,
        completion_time: Timestamp,
    ) -> Result<(), GasStoreErrors> {
        let mut time_slice = self.redelegation_queue_time_slice(ctx, completion_time)?;
        let dvv_triplet = DvvTriplet {
            del_addr: redelegation.delegator_address.clone(),
            val_src_addr: redelegation.validator_src_address.clone(),
            val_dst_addr: redelegation.validator_dst_address.clone(),
        };
        if time_slice.is_empty() {
            self.set_redelegation_queue_time_slice(ctx, completion_time, vec![dvv_triplet])?;
        } else {
            time_slice.push(dvv_triplet);
            self.set_redelegation_queue_time_slice(ctx, completion_time, time_slice)?;
        }
        Ok(())
    }

    pub fn redelegation_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        completion_time: Timestamp,
    ) -> Result<Vec<DvvTriplet>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(REDELEGATION_QUEUE_KEY);

        let key = completion_time.format_bytes_rounded(); //TODO: check if this is correct
        if let Some(bytes) = store.get(&key)? {
            Ok(serde_json::from_slice(&bytes).unwrap_or_corrupt())
        } else {
            Ok(vec![])
        }
    }

    pub fn set_redelegation_queue_time_slice<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        completion_time: Timestamp,
        redelegations: Vec<DvvTriplet>,
    ) -> Result<(), GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(REDELEGATION_QUEUE_KEY);

        let key = completion_time.format_bytes_rounded();
        let value = DvvTriplets::from(redelegations).encode_vec();
        store.set(key, value)
    }

    /// Returns a concatenated list of all the timeslices inclusively previous to
    /// currTime, and deletes the timeslices from the queue
    pub fn dequeue_all_mature_redelegation_queue<
        DB: Database,
        CTX: InfallibleContextMut<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        time: &Timestamp,
    ) -> Vec<DvvTriplet> {
        let (keys, mature_redelegations) = {
            let storage = InfallibleContext::infallible_store(ctx, &self.store_key);
            let store = storage.prefix_store(REDELEGATION_QUEUE_KEY);

            // gets an iterator for all timeslices from time 0 until the current Blockheader time
            let end = redelegation_time_key(time);
            let mut mature_redelegations = vec![];
            let mut keys = vec![];
            for (k, v) in store.into_range(..=end) {
                let time_slice: Vec<DvvTriplet> = DvvTriplets::decode::<Bytes>(v.to_vec().into())
                    .unwrap_or_corrupt()
                    .triplets;
                mature_redelegations.extend(time_slice);
                keys.push(k.to_vec());
            }
            (keys, mature_redelegations)
        };

        let storage = InfallibleContextMut::infallible_store_mut(ctx, &self.store_key);
        let mut store = storage.prefix_store_mut(REDELEGATION_QUEUE_KEY);
        keys.iter().for_each(|k| {
            store.delete(k);
        });
        mature_redelegations
    }
}
