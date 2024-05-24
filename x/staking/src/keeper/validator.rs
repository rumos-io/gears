pub use super::*;
use crate::{
    consts::expect::{
        SERDE_DECODING_DOMAIN_TYPE, SERDE_ENCODING_DOMAIN_TYPE, TIMESTAMP_NANOS_EXPECT,
    },
    Commission, CreateValidator, Validator,
};
use gears::{types::address::ConsAddress, types::context::tx::TxContext};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    /// CreateValidator defines a method for creating a new validator
    pub fn create_validator<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &CreateValidator,
    ) -> Result<(), AppError> {
        let params = self.staking_params_keeper.get(&ctx.multi_store());

        if self.validator(ctx, &msg.validator_address).is_some() {
            return Err(AppError::Custom(format!(
                "Account {} exists",
                msg.validator_address
            )));
        };

        let cons_addr: ConsAddress = msg.pub_key.clone().into();
        if self.validator_by_cons_addr(ctx, &cons_addr).is_ok() {
            return Err(AppError::Custom(format!(
                "Public key {} exists",
                ConsAddress::from(msg.pub_key.clone())
            )));
        }

        if msg.value.denom != params.bond_denom {
            return Err(AppError::InvalidRequest(format!(
                "invalid coin denomination: got {}, expected {}",
                msg.value.denom, params.bond_denom
            )));
        }

        msg.description.ensure_length()?;

        let consensus_validators = &ctx.consensus_params().validator;
        if let Some(consensus_validators) = consensus_validators {
            // TODO: discuss impl of `str_type`
            let pub_key_type = msg.pub_key.str_type();
            if !consensus_validators
                .pub_key_types
                .iter()
                .any(|key_type| pub_key_type == key_type)
            {
                return Err(AppError::InvalidPublicKey);
            }
        }

        let mut validator = Validator::new_with_defaults(
            msg.validator_address.clone(),
            msg.pub_key.clone(),
            msg.description.clone(),
        );

        let update_time = ctx.get_time().ok_or(AppError::TxValidation(
            "Transaction doesn't have valid timestamp.".to_string(),
        ))?;
        let commission = Commission {
            commission_rates: msg.commission.clone(),
            update_time,
        };

        validator.set_initial_commission(commission)?;
        validator.min_self_delegation = msg.min_self_delegation;

        self.set_validator(ctx, &validator);
        self.set_validator_by_cons_addr(ctx, &validator);
        self.set_new_validator_by_power_index(ctx, &validator);

        // call the after-creation hook
        self.after_validator_created(ctx, &validator);

        // move coins from the msg.address account to a (self-delegation) delegator account
        // the validator account and global shares are updated within here
        // NOTE source will always be from a wallet which are unbonded
        self.delegate(
            ctx,
            msg.delegator_address.clone(),
            msg.value.amount,
            BondStatus::Unbonded,
            &mut validator,
            true,
        )?;

        ctx.append_events(vec![
            Event {
                r#type: EVENT_TYPE_CREATE_VALIDATOR.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: msg.validator_address.to_string().as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.as_bytes().into(),
                        value: serde_json::to_vec(&msg.value)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                ],
            },
            Event {
                r#type: EVENT_TYPE_MESSAGE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_MODULE.as_bytes().into(),
                        value: ATTRIBUTE_VALUE_CATEGORY.as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_SENDER.as_bytes().into(),
                        value: msg.delegator_address.to_string().as_bytes().to_vec().into(),
                        index: false,
                    },
                ],
            },
        ]);

        Ok(())
    }

    pub fn validator<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        key: &ValAddress,
    ) -> Option<Validator> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(VALIDATORS_KEY);
        validators_store
            .get(key.to_string().as_bytes())
            .map(|e| serde_json::from_slice(&e).expect(SERDE_DECODING_DOMAIN_TYPE))
    }

    pub fn set_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.set(
            validator.operator_address.to_string().as_bytes().to_vec(),
            serde_json::to_vec(&validator).expect(SERDE_ENCODING_DOMAIN_TYPE),
        );
    }

    pub fn validator_by_cons_addr<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &ConsAddress,
    ) -> anyhow::Result<Validator> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(VALIDATORS_BY_CONS_ADDR_KEY);

        if let Some(bytes) = validators_store.get(addr.to_string().as_bytes()) {
            Ok(serde_json::from_slice(&bytes)?)
        } else {
            Err(anyhow::Error::from(serde_json::Error::custom(
                "Validator doesn't exists.".to_string(),
            )))
        }
    }

    pub fn set_validator_by_cons_addr<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_BY_CONS_ADDR_KEY);

        validators_store.set(
            validator.cons_addr().to_string().as_bytes().to_vec(),
            serde_json::to_vec(&validator).expect(SERDE_ENCODING_DOMAIN_TYPE),
        );
    }

    pub fn remove_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &[u8],
    ) -> Option<Vec<u8>> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.delete(addr)
    }

    /// Update the tokens of an existing validator, update the validators power index key
    pub fn add_validator_tokens_and_shares<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
        tokens_amount: Uint256,
    ) -> Decimal256 {
        self.delete_validator_by_power_index(ctx, validator);
        let added_shares = validator.add_tokens_from_del(tokens_amount);
        self.set_validator(ctx, validator);
        self.set_validator_by_power_index(ctx, validator);
        added_shares
    }

    pub fn validator_queue_map<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        block_time: chrono::DateTime<Utc>,
        block_height: i64,
    ) -> HashMap<Vec<u8>, Vec<String>> {
        let store = ctx.kv_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATORS_QUEUE_KEY);

        let end = validator_queue_key(block_time, block_height);

        let mut res = HashMap::new();

        let mut previous_was_end = false;
        for (k, v) in iterator.range(..).take_while(|(k, _)| {
            let is_not_end = **k != end;
            let ret_res = is_not_end && !previous_was_end;
            previous_was_end = !is_not_end;
            ret_res
        }) {
            // TODO
            res.insert(
                k.to_vec(),
                serde_json::from_slice(&v).expect(SERDE_DECODING_DOMAIN_TYPE),
            );
        }
        res
    }

    pub fn delete_validator_queue<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        let addrs =
            self.unbonding_validators(ctx, &validator.unbonding_time, validator.unbonding_height);
        let val_addr = validator.operator_address.to_string();
        let new_addrs = addrs
            .into_iter()
            .filter(|addr| val_addr != **addr)
            .collect::<Vec<_>>();
        if new_addrs.is_empty() {
            self.delete_validator_queue_time_slice(
                ctx,
                validator.unbonding_time.clone(),
                validator.unbonding_height,
            );
        } else {
            self.set_unbonding_validators_queue(
                ctx,
                validator.unbonding_time.clone(),
                validator.unbonding_height,
                new_addrs,
            );
        }
        Ok(())
    }

    /// get the last validator set
    pub fn last_validators_by_addr<DB: Database, CTX: QueryableContext<DB, SK>>(
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

pub(super) fn validator_queue_key(end_time: chrono::DateTime<Utc>, end_height: i64) -> Vec<u8> {
    let height_bz = end_height.to_ne_bytes();
    let time_bz = end_time
        .timestamp_nanos_opt()
        .expect(TIMESTAMP_NANOS_EXPECT)
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
