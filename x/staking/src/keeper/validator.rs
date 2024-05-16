pub use super::*;
use crate::{
    consts::expect::{SERDE_DECODING_DOMAIN_TYPE, SERDE_ENCODING_DOMAIN_TYPE},
    Commission, CreateValidator, Validator,
};
use gears::{
    tendermint::types::proto::crypto::PublicKey, types::address::ConsAddress,
    types::context::tx::TxContext,
};

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
        msg: CreateValidator,
    ) -> Result<(), AppError> {
        let params = self.staking_params_keeper.get(&ctx.multi_store());
        let _val_by_val_addr = self
            .validator(ctx, &msg.validator_address)
            .expect("validator in the store was not found");

        let cons_addr: ConsAddress = msg.pub_key.clone().into();
        let _val_by_cons_addr = self
            .validator_by_cons_addr(ctx, &cons_addr)
            .map_err(|_e| AppError::AccountNotFound)?;
        if msg.value.denom != params.bond_denom {
            return Err(AppError::InvalidRequest(format!(
                "invalid coin denomination: got {}, expected {}",
                msg.value.denom, params.bond_denom
            )));
        }

        msg.description.ensure_length()?;

        let consensus_validators = self
            .staking_params_keeper
            .consensus_validator(&ctx.multi_store());
        if let Ok(consensus_validators) = consensus_validators {
            // TODO: implement method on new type
            let pub_key_type = match &msg.pub_key {
                PublicKey::Secp256k1(_) => "secp256k1",
                PublicKey::Ed25519(_) => "ed25519",
            };
            if !consensus_validators
                .pub_key_types
                .iter()
                .any(|key_type| pub_key_type == key_type)
            {
                return Err(AppError::InvalidPublicKey);
            }
        }

        let mut _validator =
            Validator::new_with_defaults(msg.validator_address, msg.pub_key, msg.description);

        let update_time = ctx.header().time.clone().ok_or(AppError::TxValidation(
            "Transaction doesn't have valid timestamp.".to_string(),
        ))?;
        let _commision = Commission {
            commission_rates: msg.commission,
            update_time,
        };
        todo!()
    }
    // func (k msgServer) CreateValidator(goCtx context.Context, msg *types.MsgCreateValidator) (*types.MsgCreateValidatorResponse, error) {
    //
    //     validator, err := types.NewValidator(valAddr, pk, msg.Description)
    //     if err != nil {
    //         return nil, err
    //     }
    //     commission := types.NewCommissionWithTime(
    //         msg.Commission.Rate, msg.Commission.MaxRate,
    //         msg.Commission.MaxChangeRate, ctx.BlockHeader().Time,
    //     )
    //
    //     validator, err = validator.SetInitialCommission(commission)
    //     if err != nil {
    //         return nil, err
    //     }
    //
    //     delegatorAddress, err := sdk.AccAddressFromBech32(msg.DelegatorAddress)
    //     if err != nil {
    //         return nil, err
    //     }
    //
    //     validator.MinSelfDelegation = msg.MinSelfDelegation
    //
    //     k.SetValidator(ctx, validator)
    //     k.SetValidatorByConsAddr(ctx, validator)
    //     k.SetNewValidatorByPowerIndex(ctx, validator)
    //
    //     // call the after-creation hook
    //     k.AfterValidatorCreated(ctx, validator.GetOperator())
    //
    //     // move coins from the msg.Address account to a (self-delegation) delegator account
    //     // the validator account and global shares are updated within here
    //     // NOTE source will always be from a wallet which are unbonded
    //     _, err = k.Keeper.Delegate(ctx, delegatorAddress, msg.Value.Amount, types.Unbonded, validator, true)
    //     if err != nil {
    //         return nil, err
    //     }
    //
    //     ctx.EventManager().EmitEvents(sdk.Events{
    //         sdk.NewEvent(
    //             types.EventTypeCreateValidator,
    //             sdk.NewAttribute(types.AttributeKeyValidator, msg.ValidatorAddress),
    //             sdk.NewAttribute(sdk.AttributeKeyAmount, msg.Value.String()),
    //         ),
    //         sdk.NewEvent(
    //             sdk.EventTypeMessage,
    //             sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
    //             sdk.NewAttribute(sdk.AttributeKeySender, msg.DelegatorAddress),
    //         ),
    //     })
    //
    //     return &types.MsgCreateValidatorResponse{}, nil
    // }

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
    ) -> anyhow::Result<()> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut validators_store = store.prefix_store_mut(VALIDATORS_KEY);
        validators_store.delete(addr);
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
            let mut k = validator_queue_key(block_time, block_height);
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
        let addrs =
            self.unbonding_validators(ctx, &validator.unbonding_time, validator.unbonding_height)?;
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
        for (k, v) in store.range(..) {
            let k: ValAddress = serde_json::from_slice(&k).expect(SERDE_DECODING_DOMAIN_TYPE);
            last.insert(k.to_string(), v.to_vec());
        }
        last
    }
}

pub(super) fn validator_queue_key(end_time: chrono::DateTime<Utc>, end_height: i64) -> Vec<u8> {
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
