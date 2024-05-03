pub use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
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
}
