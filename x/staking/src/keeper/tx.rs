use super::*;
use crate::{
    Commission, CreateValidator, DelegateMsg, EditValidator, RedelegateMsg, UndelegateMsg,
};
use gears::{
    baseapp::ValidatorParams, context::tx::TxContext, extensions::corruption::UnwrapCorrupt,
    types::address::ConsAddress,
};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: StakingBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    /// create_validator defines a method for creating a new validator
    pub fn create_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        consensus_validators: ValidatorParams,
        msg: &CreateValidator,
    ) -> Result<(), anyhow::Error> {
        let params = self.staking_params_keeper.try_get(ctx)?;

        if self.validator(ctx, &msg.validator_address)?.is_some() {
            return Err(anyhow::anyhow!("Account {} exists", msg.validator_address));
        };

        let cons_addr: ConsAddress = msg.pubkey.clone().into();
        if self.validator_by_cons_addr(ctx, &cons_addr)?.is_some() {
            return Err(anyhow::anyhow!(
                "Public key {} exists",
                ConsAddress::from(msg.pubkey.clone())
            ));
        }

        if &msg.value.denom != params.bond_denom() {
            return Err(anyhow::anyhow!(
                "invalid coin denomination: got {}, expected {}",
                msg.value.denom,
                params.bond_denom()
            ));
        }

        msg.description.ensure_length()?;

        let pub_key_type = msg.pubkey.str_type();
        if !consensus_validators
            .pub_key_types
            .iter()
            .any(|key_type| pub_key_type == key_type)
        {
            return Err(anyhow::anyhow!("invalid public key"));
        }

        let mut validator = Validator::new_with_defaults(
            msg.validator_address.clone(),
            msg.pubkey.clone(),
            msg.description.clone(),
        );

        let update_time = ctx.get_time();
        let commission = Commission::new(msg.commission.clone(), update_time);
        validator.set_initial_commission(commission);
        validator.min_self_delegation = msg.min_self_delegation;

        self.set_validator(ctx, &validator)?;
        self.set_validator_by_cons_addr(ctx, &validator)?;
        self.set_new_validator_by_power_index(ctx, &validator)?;

        // call the after-creation hook
        self.after_validator_created(ctx, &validator);

        // move coins from the msg.address account to a (self-delegation) delegator account
        // the validator account and global shares are updated within here
        // NOTE source will always be from a wallet which are unbonded
        self.delegate(
            ctx,
            &msg.delegator_address,
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
                        key: ATTRIBUTE_KEY_VALIDATOR.into(),
                        value: msg.validator_address.to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.into(),
                        value: serde_json::to_string(&msg.value)
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
                        key: ATTRIBUTE_KEY_MODULE.into(),
                        value: ATTRIBUTE_VALUE_CATEGORY.into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_SENDER.into(),
                        value: msg.delegator_address.to_string().into(),
                        index: false,
                    },
                ],
            },
        ]);

        Ok(())
    }

    /// edit_validator defines a method for editing an existing validator
    pub fn edit_validator<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &EditValidator,
    ) -> Result<(), anyhow::Error> {
        // validator must already be registered
        let mut validator = self
            .validator(ctx, &msg.validator_address)?
            .ok_or(anyhow::anyhow!(
                "Account not {} exists",
                msg.validator_address
            ))?;

        // replace all editable fields (clients should autofill existing values)
        let description = validator
            .description
            .create_updated_description(&msg.description)?;
        validator.description = description;

        if let Some(rate) = msg.commission_rate {
            let commission = self
                .create_updated_validator_commission(ctx, &validator, rate)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            // call the before-modification hook since we're about to update the commission
            self.before_validator_modified(ctx, &validator);
            validator.commission = commission;
        }

        if let Some(min_self_delegation) = msg.min_self_delegation {
            if min_self_delegation <= validator.min_self_delegation {
                return Err(anyhow::anyhow!(
                    "trying to decrease validator minimal self delegation",
                ));
            }

            if min_self_delegation > validator.tokens {
                return Err(anyhow::anyhow!(
                    "validator has not enough tokens to delegate"
                ));
            }

            validator.min_self_delegation = min_self_delegation;
        }

        self.set_validator(ctx, &validator)?;

        ctx.append_events(vec![
            Event {
                r#type: EVENT_TYPE_EDIT_VALIDATOR.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.into(),
                        value: msg.validator_address.to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_COMMISSION_RATE.into(),
                        value: serde_json::to_string(&validator.commission)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_MIN_SELF_DELEGATION.into(),
                        value: validator.min_self_delegation.to_string().into(),
                        index: false,
                    },
                ],
            },
            Event {
                r#type: EVENT_TYPE_MESSAGE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_MODULE.into(),
                        value: ATTRIBUTE_VALUE_CATEGORY.into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_SENDER.into(),
                        value: msg.validator_address.to_string().into(),
                        index: false,
                    },
                ],
            },
        ]);

        Ok(())
    }

    /// delegate_cmd_handler defines a method for performing a delegation of coins from a delegator to a validator
    pub fn delegate_cmd_handler<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &DelegateMsg,
    ) -> Result<(), anyhow::Error> {
        let mut validator = if let Some(validator) = self.validator(ctx, &msg.validator_address)? {
            validator
        } else {
            return Err(anyhow::anyhow!("account not found"));
        };
        let params = self.staking_params_keeper.try_get(ctx)?;
        let delegator_address = msg.delegator_address.clone();

        if &msg.amount.denom != params.bond_denom() {
            return Err(anyhow::anyhow!(
                "invalid coin denomination: got {}, expected {}",
                msg.amount.denom,
                params.bond_denom()
            ));
        }

        // NOTE: source funds are always unbonded
        let new_shares = self.delegate(
            ctx,
            &delegator_address,
            msg.amount.amount,
            BondStatus::Unbonded,
            &mut validator,
            true,
        )?;

        ctx.append_events(vec![
            Event {
                r#type: EVENT_TYPE_DELEGATE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.into(),
                        value: msg.validator_address.to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.into(),
                        value: serde_json::to_string(&msg.amount)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_NEW_SHARES.into(),
                        value: new_shares.to_string().into(),
                        index: false,
                    },
                ],
            },
            Event {
                r#type: EVENT_TYPE_MESSAGE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_MODULE.into(),
                        value: ATTRIBUTE_VALUE_CATEGORY.into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_SENDER.into(),
                        value: msg.delegator_address.to_string().into(),
                        index: false,
                    },
                ],
            },
        ]);

        Ok(())
    }

    /// redelegate_cmd_handler defines a method for performing a redelegation of coins from a delegator and source validator to a destination validator
    pub fn redelegate_cmd_handler<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &RedelegateMsg,
    ) -> Result<(), anyhow::Error> {
        let shares = self
            .validate_unbond_amount(
                ctx,
                &msg.delegator_address,
                &msg.src_validator_address,
                msg.amount.amount,
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let params = self.staking_params_keeper.try_get(ctx)?;

        if &msg.amount.denom != params.bond_denom() {
            return Err(anyhow::anyhow!(
                "invalid coin denomination: got {}, expected {}",
                msg.amount.denom,
                params.bond_denom()
            ));
        }

        let completion_time = self
            .begin_redelegation(
                ctx,
                &msg.delegator_address,
                &msg.src_validator_address,
                &msg.dst_validator_address,
                shares,
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        ctx.append_events(vec![
            Event {
                r#type: EVENT_TYPE_REDELEGATE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_SRC_VALIDATOR.into(),
                        value: msg.src_validator_address.to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_DST_VALIDATOR.into(),
                        value: msg.dst_validator_address.to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.into(),
                        value: serde_json::to_string(&msg.amount)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_COMPLETION_TIME.into(),
                        value: serde_json::to_string(&completion_time)
                            .unwrap_or_corrupt()
                            .into(),
                        index: false,
                    },
                ],
            },
            Event {
                r#type: EVENT_TYPE_MESSAGE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_MODULE.into(),
                        value: ATTRIBUTE_VALUE_CATEGORY.into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_SENDER.into(),
                        value: msg.delegator_address.to_string().into(),
                        index: false,
                    },
                ],
            },
        ]);

        Ok(())
    }

    /// undelegate_cmd_handler defines a method for performing an undelegation from a delegate and a validator
    pub fn undelegate_cmd_handler<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &UndelegateMsg,
    ) -> Result<(), anyhow::Error> {
        let shares = self
            .validate_unbond_amount(
                ctx,
                &msg.delegator_address,
                &msg.validator_address,
                msg.amount.amount,
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let params = self.staking_params_keeper.try_get(ctx)?;
        if &msg.amount.denom != params.bond_denom() {
            return Err(anyhow::anyhow!(
                "invalid coin denomination: got {}, expected {}",
                msg.amount.denom,
                params.bond_denom()
            ));
        }

        let completion_time = self
            .undelegate(ctx, &msg.delegator_address, &msg.validator_address, shares)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        ctx.append_events(vec![
            Event {
                r#type: EVENT_TYPE_UNBOND.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.into(),
                        value: msg.validator_address.to_string().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.into(),
                        value: serde_json::to_string(&msg.amount)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_COMPLETION_TIME.into(),
                        // TODO: format time
                        value: serde_json::to_string(&completion_time)
                            .unwrap_or_corrupt()
                            .into(),
                        index: false,
                    },
                ],
            },
            Event {
                r#type: EVENT_TYPE_MESSAGE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_MODULE.into(),
                        value: ATTRIBUTE_VALUE_CATEGORY.into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_SENDER.into(),
                        value: msg.delegator_address.to_string().into(),
                        index: false,
                    },
                ],
            },
        ]);

        Ok(())
    }
}
