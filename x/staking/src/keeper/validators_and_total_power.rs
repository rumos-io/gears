use super::*;
use gears::error::IBC_ENCODE_UNWRAP;
use gears::tendermint::types::proto::Protobuf;
use gears::{
    context::InfallibleContext, store::database::ext::UnwrapCorrupt,
    types::base::coin::Uint256Proto,
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
    /// Load the last total validator power.
    pub fn last_total_power<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Option<Uint256> {
        let store = InfallibleContext::infallible_store(ctx, &self.store_key);
        store.get(&LAST_TOTAL_POWER_KEY).map(|bytes| {
            Uint256::from_be_bytes(bytes.try_into().expect(
                "The method from_be_bytes accepts array of bytes.
                The store returns owned value of stored array.
                Error can happen when vector has invalid length.
                Please, check the store methods",
            ))
        })
    }

    pub fn set_last_total_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        last_total_power: Uint256,
    ) -> Result<(), GasStoreErrors> {
        let mut store = TransactionalContext::kv_store_mut(ctx, &self.store_key);
        let val = Uint256Proto {
            uint: last_total_power,
        }
        .encode_vec()
        .expect(IBC_ENCODE_UNWRAP); // TODO:IBC;
        store.set(LAST_TOTAL_POWER_KEY, val)
    }

    /// get the last validator set
    pub fn validators_power_store_vals_map<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> anyhow::Result<HashMap<Vec<u8>, ValAddress>> {
        let store = ctx.kv_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATORS_BY_POWER_INDEX_KEY);
        let mut res = HashMap::new();
        // TODO:D Handle error if you need
        for next in iterator.into_range(..) {
            let (k, v) = next?;
            res.insert(k.to_vec(), ValAddress::try_from(v.to_vec())?);
        }
        Ok(res)
    }

    pub fn set_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<(), GasStoreErrors> {
        let power_reduction = self.power_reduction(ctx);
        let store = TransactionalContext::kv_store_mut(ctx, &self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_BY_POWER_INDEX_KEY);

        // jailed validators are not kept in the power index
        if validator.jailed {
            return Ok(());
        }

        validators_store.set(
            validator.key_by_power_index_key(power_reduction),
            Vec::from(validator.operator_address.clone()),
        )
    }

    pub fn set_new_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<(), GasStoreErrors> {
        let power_reduction = self.power_reduction(ctx);
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_BY_POWER_INDEX_KEY);

        validators_store.set(
            validator.key_by_power_index_key(power_reduction),
            validator.operator_address.to_string().as_bytes().to_vec(),
        )
    }

    pub fn delete_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        let power_reduction = self.power_reduction(ctx);
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(VALIDATORS_BY_POWER_INDEX_KEY);
        store.delete(&validator.key_by_power_index_key(power_reduction))
    }

    /// get the last validator set
    pub fn last_validators_by_addr<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> HashMap<String, Vec<u8>> {
        let mut last = HashMap::new();
        let store = ctx.infallible_store(&self.store_key);
        let store = store.prefix_store(LAST_VALIDATOR_POWER_KEY);
        for (k, v) in store.into_range(..) {
            let k: ValAddress = serde_json::from_slice(&k).unwrap_or_corrupt();
            last.insert(k.to_string(), v.to_vec());
        }
        last
    }

    /// get the group of the bonded validators
    pub fn last_validators<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Vec<Validator>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(LAST_VALIDATOR_POWER_KEY);

        // add the actual validator power sorted store
        let max_validators = self.staking_params_keeper.try_get(ctx)?.max_validators as usize;
        let mut validators = Vec::with_capacity(max_validators);
        for (i, next) in validators_store.into_range(..).enumerate() {
            let (_k, v) = next?;
            assert!(
                i < max_validators,
                "more validators than maxValidators found"
            );
            let last_validator: LastValidatorPower = serde_json::from_slice(&v).unwrap_or_corrupt();
            let validator = self
                .validator(ctx, &last_validator.address)?
                .expect("validator stored in last validators queue should be presented in store");
            validators.push(validator);
        }
        Ok(validators)
    }

    pub fn set_last_validator_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &LastValidatorPower,
    ) -> Result<(), GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(LAST_VALIDATOR_POWER_KEY);
        let key = validator.address.to_string().as_bytes().to_vec();
        delegations_store.set(
            key,
            serde_json::to_vec(&validator).expect(SERDE_ENCODING_DOMAIN_TYPE),
        )
    }

    pub fn delete_last_validator_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &ValAddress,
    ) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut delegations_store = store.prefix_store_mut(LAST_VALIDATOR_POWER_KEY);
        delegations_store.delete(validator.to_string().as_bytes())
    }
}
