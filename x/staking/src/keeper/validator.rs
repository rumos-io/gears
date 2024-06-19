use super::*;
use crate::{consts::error::SERDE_ENCODING_DOMAIN_TYPE, validator_queue_key, Validator};
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
    ) -> Result<Decimal256, GasStoreErrors> {
        self.delete_validator_by_power_index(ctx, validator)?;
        let added_shares = validator.add_tokens_from_del(tokens_amount);
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
    ) -> Result<Uint256, GasStoreErrors> {
        self.delete_validator_by_power_index(ctx, validator)?;
        let removed_tokens = validator.remove_del_shares(shares_to_remove);
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;
        Ok(removed_tokens)
    }

    pub fn validator_queue_map<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        block_time: chrono::DateTime<Utc>,
        block_height: u64,
    ) -> HashMap<Vec<u8>, Vec<String>> {
        let store = ctx.infallible_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATOR_QUEUE_KEY);

        let mut res = HashMap::new();

        let end = validator_queue_key(block_time, block_height);
        let mut previous_was_end = false;
        for (k, v) in iterator.into_range(..).take_while(|(k, _)| {
            let is_not_end = **k != end;
            let ret_res = is_not_end && !previous_was_end;
            previous_was_end = !is_not_end;
            ret_res
        }) {
            // TODO
            res.insert(k.to_vec(), serde_json::from_slice(&v).unwrap_or_corrupt());
        }
        res
    }

    pub fn delete_validator_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), GasStoreErrors> {
        let addrs =
            self.unbonding_validators(ctx, &validator.unbonding_time, validator.unbonding_height);
        let val_addr = validator.operator_address.to_string();
        let new_addrs = addrs?
            .into_iter()
            .filter(|addr| val_addr != **addr)
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
