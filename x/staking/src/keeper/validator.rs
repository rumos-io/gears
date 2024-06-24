use super::*;
use crate::{consts::error::SERDE_ENCODING_DOMAIN_TYPE, Validator};
use gears::{store::database::ext::UnwrapCorrupt, types::address::ConsAddress};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    pub fn validator<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        key: &ValAddress,
    ) -> Result<Option<Validator>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(VALIDATORS_KEY);
        Ok(validators_store
            .get(key.to_string().as_bytes())?
            .map(|e| serde_json::from_slice(&e).unwrap_or_corrupt()))
    }

    pub fn set_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<(), GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.set(
            validator.operator_address.to_string().as_bytes().to_vec(),
            serde_json::to_vec(&validator).expect(SERDE_ENCODING_DOMAIN_TYPE),
        )
    }

    pub fn remove_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.delete(validator.operator_address.to_string().as_bytes())
    }

    pub fn jail_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), GasStoreErrors> {
        assert!(
            !validator.jailed,
            "cannot jail already jailed validator, validator: {}",
            validator.operator_address
        );
        validator.jailed = true;
        self.set_validator(ctx, validator)?;
        self.delete_validator_by_power_index(ctx, validator)?;
        Ok(())
    }

    pub fn validator_by_cons_addr<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &ConsAddress,
    ) -> Result<Option<Validator>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(VALIDATORS_BY_CONS_ADDR_KEY);

        Ok(validators_store
            .get(addr.to_string().as_bytes())?
            .map(|bytes| serde_json::from_slice(&bytes).unwrap_or_corrupt()))
    }

    pub fn set_validator_by_cons_addr<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<(), GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_BY_CONS_ADDR_KEY);

        validators_store.set(
            validator.cons_addr().to_string().as_bytes().to_vec(),
            serde_json::to_vec(&validator).expect(SERDE_ENCODING_DOMAIN_TYPE),
        )
    }

    /// Update the tokens of an existing validator, update the validators power index key
    pub fn add_validator_tokens_and_shares<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
        tokens_amount: Uint256,
    ) -> anyhow::Result<Decimal256> {
        self.delete_validator_by_power_index(ctx, validator)?;
        let added_shares = validator.add_tokens_from_del(tokens_amount)?;
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;
        Ok(added_shares)
    }

    /// Update the tokens of an existing validator, update the validators power index key
    pub fn remove_validator_tokens_and_shares<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
        shares_to_remove: Decimal256,
    ) -> anyhow::Result<Uint256> {
        self.delete_validator_by_power_index(ctx, validator)?;
        let removed_tokens = validator.remove_del_shares(shares_to_remove)?;
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;
        Ok(removed_tokens)
    }
}
