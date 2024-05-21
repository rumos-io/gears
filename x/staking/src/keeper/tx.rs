use crate::{Commission, CreateValidator, DelegateMsg};
use gears::types::{address::ConsAddress, context::tx::TxContext};

pub use super::*;

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

        let consensus_validators = self
            .staking_params_keeper
            .consensus_validator(&ctx.multi_store());
        if let Ok(consensus_validators) = consensus_validators {
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

        let update_time = ctx
            .header()
            .expect("TxContext always has context")
            .time
            .clone()
            .ok_or(AppError::TxValidation(
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

    /// Delegate defines a method for performing a delegation of coins from a delegator to a validator
    pub fn delegate_cmd_handler<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &DelegateMsg,
    ) -> Result<(), AppError> {
        let mut validator = if let Some(validator) = self.validator(ctx, &msg.validator_address) {
            validator
        } else {
            return Err(AppError::AccountNotFound);
        };
        let params = self.staking_params_keeper.get(&ctx.multi_store());
        let delegator_address = msg.delegator_address.clone();

        if msg.amount.denom != params.bond_denom {
            return Err(AppError::InvalidRequest(format!(
                "invalid coin denomination: got {}, expected {}",
                msg.amount.denom, params.bond_denom
            )));
        }

        // NOTE: source funds are always unbonded
        let new_shares = self.delegate(
            ctx,
            delegator_address,
            msg.amount.amount,
            BondStatus::Unbonded,
            &mut validator,
            true,
        )?;

        // TODO
        // if msg.Amount.Amount.IsInt64() {
        //     defer func() {
        //         telemetry.IncrCounter(1, types.ModuleName, "delegate")
        //         telemetry.SetGaugeWithLabels(
        //             []string{"tx", "msg", msg.Type()},
        //             float32(msg.Amount.Amount.Int64()),
        //             []metrics.Label{telemetry.NewLabel("denom", msg.Amount.Denom)},
        //         )
        //     }()
        // }

        ctx.append_events(vec![
            Event {
                r#type: EVENT_TYPE_DELEGATE.to_string(),
                attributes: vec![
                    EventAttribute {
                        key: ATTRIBUTE_KEY_VALIDATOR.as_bytes().into(),
                        value: msg.validator_address.to_string().as_bytes().to_vec().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_AMOUNT.as_bytes().into(),
                        value: serde_json::to_vec(&msg.amount)
                            .expect(SERDE_ENCODING_DOMAIN_TYPE)
                            .into(),
                        index: false,
                    },
                    EventAttribute {
                        key: ATTRIBUTE_KEY_NEW_SHARES.as_bytes().into(),
                        value: new_shares.to_string().as_bytes().to_vec().into(),
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
}
