pub use super::*;
use crate::{
    consts::error::{SERDE_ENCODING_DOMAIN_TYPE, TIMESTAMP_NANOS_EXPECT},
    Validator,
};
use gears::{store::database::ext::UnwrapCorrupt, types::address::ConsAddress};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn validator<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        key: &ValAddress,
    ) -> Result<Option<Validator>, StoreErrors> {
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
    ) -> Result<(), StoreErrors> {
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
    ) -> Result<Option<Vec<u8>>, StoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.delete(validator.operator_address.to_string().as_bytes())
    }

    pub fn jail_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), StoreErrors> {
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
    ) -> Result<Option<Validator>, StoreErrors> {
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
    ) -> Result<(), StoreErrors> {
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
    ) -> Result<Decimal256, StoreErrors> {
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
    ) -> Result<Uint256, StoreErrors> {
        self.delete_validator_by_power_index(ctx, validator)?;
        let removed_tokens = validator.remove_del_shares(shares_to_remove);
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;
        Ok(removed_tokens)
    }

    pub fn validator_queue_map<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        block_time: chrono::DateTime<Utc>,
        block_height: u64,
    ) -> HashMap<Vec<u8>, Vec<String>> {
        let store = ctx.kv_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATORS_QUEUE_KEY);

        let end = validator_queue_key(block_time, block_height);

        let mut res = HashMap::new();

        let mut previous_was_end = false;
        // TODO:D Handle error if you need
        for (k, v) in iterator
            .range(..)
            .to_infallible_iter()
            .take_while(|(k, _)| {
                let is_not_end = **k != end;
                let ret_res = is_not_end && !previous_was_end;
                previous_was_end = !is_not_end;
                ret_res
            })
        {
            // TODO
            res.insert(k.to_vec(), serde_json::from_slice(&v).unwrap_or_corrupt());
        }
        res
    }

    pub fn delete_validator_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), StoreErrors> {
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

    /// get the last validator set
    pub fn last_validators_by_addr<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> HashMap<String, Vec<u8>> {
        let mut last = HashMap::new();
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(LAST_VALIDATOR_POWER_KEY);
        // TODO:D Handle error if you need
        for (k, v) in store.range(..).to_infallible_iter() {
            let k: ValAddress = serde_json::from_slice(&k).unwrap_or_corrupt();
            last.insert(k.to_string(), v.to_vec());
        }
        last
    }
}

pub(super) fn validator_queue_key(end_time: chrono::DateTime<Utc>, end_height: u64) -> Vec<u8> {
    let height_bz = end_height.to_le_bytes();
    let time_bz = end_time
        .timestamp_nanos_opt()
        .expect(TIMESTAMP_NANOS_EXPECT)
        .to_le_bytes();

    let mut bz = VALIDATORS_QUEUE_KEY.to_vec();
    bz.extend_from_slice(&(time_bz.len() as u64).to_le_bytes());
    bz.extend_from_slice(&time_bz);
    bz.extend_from_slice(&height_bz);
    bz
}

pub(super) fn parse_validator_queue_key(
    key: &[u8],
) -> anyhow::Result<(chrono::DateTime<Utc>, u64)> {
    let prefix_len = VALIDATORS_QUEUE_KEY.len();
    if key[..prefix_len] != VALIDATORS_QUEUE_KEY {
        return Err(
            AppError::Custom("Invalid validators queue key. Invalid prefix.".into()).into(),
        );
    }
    let time_len = u64::from_le_bytes(key[prefix_len..prefix_len + 8].try_into()?);
    let time = chrono::DateTime::from_timestamp_nanos(i64::from_le_bytes(
        key[prefix_len + 8..prefix_len + 8 + time_len as usize].try_into()?,
    ));
    let height = u64::from_le_bytes(key[prefix_len + 8 + time_len as usize..].try_into()?);
    Ok((time, height))
}
