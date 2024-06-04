use super::*;
use crate::{historical_info_key, HistoricalInfo};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn track_historical_info<DB: Database>(&self, ctx: &mut BlockContext<'_, DB, SK>) {
        let params = self.staking_params_keeper.get(ctx);
        let entry_num = params.historical_entries;

        // Prune store to ensure we only have parameter-defined historical entries.
        // In most cases, this will involve removing a single historical entry.
        // In the rare scenario when the historical entries gets reduced to a lower value k'
        // from the original value k. k - k' entries must be deleted from the store.
        // Since the entries to be deleted are always in a continuous range, we can iterate
        // over the historical entries starting from the most recent version to be pruned
        // and then return at the first empty entry.

        if (ctx.height() as i64 - entry_num as i64) >= 0 {
            for i in (ctx.height() - entry_num as u64)..=0 {
                if let Some(_info) = self.historical_info(ctx, i) {
                    self.delete_historical_info(ctx, i);
                } else {
                    break;
                }
            }
        }

        // if there is no need to persist historicalInfo, return
        if entry_num == 0 {
            return;
        }

        // Create HistoricalInfo struct
        let last_validators = self.last_validators(ctx);
        let historical_entry = HistoricalInfo::new(
            ctx.header.clone(),
            last_validators,
            self.power_reduction(ctx),
        );

        // Set latest HistoricalInfo at current height
        self.set_historical_info(ctx, ctx.height(), &historical_entry);
    }

    /// historical_info gets the historical info at a given height
    pub fn historical_info<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        height: u64,
    ) -> Option<HistoricalInfo> {
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(HISTORICAL_INFO_KEY);
        let key = historical_info_key(height);
        store
            .get(&key)
            .map(|bytes| serde_json::from_slice(&bytes).unwrap_or_corrupt())
    }

    /// delete_historical_info deletes the historical info at a given height
    pub fn delete_historical_info<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        height: u64,
    ) -> Option<Vec<u8>> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(HISTORICAL_INFO_KEY);
        let key = historical_info_key(height);
        store.delete(&key)
    }

    /// set_historical_info deletes the historical info at a given height
    pub fn set_historical_info<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        height: u64,
        info: &HistoricalInfo,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(HISTORICAL_INFO_KEY);
        let key = historical_info_key(height);
        let info = serde_json::to_vec(&info).expect(SERDE_ENCODING_DOMAIN_TYPE);
        store.set(key, info)
    }
}
