pub use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn get_validator<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        key: &[u8],
    ) -> anyhow::Result<Validator> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(VALIDATORS_KEY);
        if let Some(e) = validators_store.get(key) {
            Ok(serde_json::from_slice(&e)?)
        } else {
            Err(anyhow::Error::from(serde_json::Error::custom(
                "Validator doesn't exists.".to_string(),
            )))
        }
    }

    pub fn set_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.set(
            validator.operator_address.to_string().as_bytes().to_vec(),
            serde_json::to_vec(&validator)?,
        );
        Ok(())
    }

    pub fn remove_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &[u8],
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.delete(addr);
        Ok(())
    }

    pub fn set_validator_by_cons_addr<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.set(
            validator
                .consensus_pubkey
                .get_address()
                .to_string()
                .encode_to_vec(),
            serde_json::to_vec(&validator)?,
        );
        Ok(())
    }

    pub fn validator_queue_iterator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        block_time: chrono::DateTime<Utc>,
        block_height: i64,
    ) -> anyhow::Result<HashMap<Vec<u8>, Vec<String>>> {
        let store = ctx.kv_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATORS_QUEUE_KEY);

        let end = {
            let mut k = get_validator_queue_key(block_time, block_height);
            k.push(0);
            k
        };

        let mut res = HashMap::new();
        for (k, v) in iterator.range(..).take_while(|(k, _)| **k != end) {
            res.insert(k.to_vec(), serde_json::from_slice(&v)?);
        }
        Ok(res)
    }

    pub fn delete_validator_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        let addrs = self.get_unbonding_validators(
            ctx,
            validator.unbonding_time,
            validator.unbonding_height,
        )?;
        let val_addr = validator.operator_address.to_string();
        let new_addrs = addrs
            .into_iter()
            .filter(|addr| val_addr != **addr)
            .collect::<Vec<_>>();
        if new_addrs.is_empty() {
            self.delete_validator_queue_time_slice(
                ctx,
                validator.unbonding_time,
                validator.unbonding_height,
            );
        } else {
            self.set_unbonding_validators_queue(
                ctx,
                validator.unbonding_time,
                validator.unbonding_height,
                new_addrs,
            )?;
        }
        Ok(())
    }

    /// get the last validator set
    pub fn get_last_validators_by_addr<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> anyhow::Result<HashMap<String, Vec<u8>>> {
        let mut last = HashMap::new();
        let store = ctx.kv_store(&self.store_key);
        let store = store.prefix_store(LAST_VALIDATOR_POWER_KEY);
        for (k, v) in store.range(..) {
            let k: ValAddress = serde_json::from_slice(&k)?;
            last.insert(k.to_string(), v.to_vec());
        }
        Ok(last)
    }
}

pub(super) fn get_validator_queue_key(end_time: chrono::DateTime<Utc>, end_height: i64) -> Vec<u8> {
    let height_bz = end_height.to_ne_bytes();
    let time_bz = end_time
        .timestamp_nanos_opt()
        .expect("The timestamp_nanos_opt produces an integer that represents time in nanoseconds.
                The error in this method means that some system failure happened and the system cannot continue work.")
        .to_ne_bytes();

    let mut bz = VALIDATORS_QUEUE_KEY.to_vec();
    bz.extend_from_slice(&(time_bz.len() as u64).to_ne_bytes());
    bz.extend_from_slice(&time_bz);
    bz.extend_from_slice(&height_bz);
    bz
}

pub(super) fn parse_validator_queue_key(
    key: &[u8],
) -> anyhow::Result<(chrono::DateTime<Utc>, i64)> {
    let prefix_len = VALIDATORS_QUEUE_KEY.len();
    if key[..prefix_len] != VALIDATORS_QUEUE_KEY {
        return Err(
            AppError::Custom("Invalid validators queue key. Invalid prefix.".into()).into(),
        );
    }
    let time_len = u64::from_ne_bytes(key[prefix_len..prefix_len + 8].try_into()?);
    let time = chrono::DateTime::from_timestamp_nanos(i64::from_ne_bytes(
        key[prefix_len + 8..prefix_len + 8 + time_len as usize].try_into()?,
    ));
    let height = i64::from_ne_bytes(key[prefix_len + 8 + time_len as usize..].try_into()?);
    Ok((time, height))
}
