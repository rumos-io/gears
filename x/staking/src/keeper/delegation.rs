pub use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
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
}
