use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        KH: KeeperHooks<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    pub fn unbond<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_addr: &ValAddress,
        shares: Decimal256,
    ) -> anyhow::Result<Uint256> {
        // check if a delegation object exists in the store
        let mut delegation = if let Some(delegation) = self.delegation(ctx, del_addr, val_addr)? {
            delegation
        } else {
            return Err(AppError::Custom("no delegator for address".to_string()).into());
        };

        // call the before-delegation-modified hook
        self.before_delegation_shares_modified(ctx, del_addr, val_addr);

        // ensure that we have enough shares to remove
        if delegation.shares < shares {
            return Err(AppError::Custom("not enough delegation shares".to_string()).into());
        }

        // get validator
        let mut validator = if let Some(validator) = self.validator(ctx, val_addr)? {
            validator
        } else {
            return Err(AppError::Custom("no validator found".to_string()).into());
        };

        // subtract shares from delegation
        delegation.shares -= shares;

        let is_validator_operator =
            Vec::from(del_addr.clone()) == Vec::from(validator.operator_address.clone());

        // If the delegation is the operator of the validator and undelegating will decrease the validator's
        // self-delegation below their minimum, we jail the validator.
        // TODO: check code `.to_uint_floor()`
        if is_validator_operator
            && !validator.jailed
            && validator
                .tokens_from_shares(delegation.shares)?
                .to_uint_floor()
                < validator.min_self_delegation
        {
            self.jail_validator(ctx, &mut validator)?;
            // TODO: panic in sdk
            validator = self.validator(ctx, &validator.operator_address)?.unwrap()
        }

        // remove the delegation
        if delegation.shares.is_zero() {
            self.remove_delegation(ctx, &delegation)?;
        } else {
            self.set_delegation(ctx, &delegation)?;
            // call the after delegation modification hook
            self.after_delegation_modified(ctx, del_addr, &delegation.validator_address);
        }

        // remove the shares and coins from the validator
        // NOTE that the amount is later (in keeper.Delegation) moved between staking module pools
        let tokens_amount = self.remove_validator_tokens_and_shares(ctx, &mut validator, shares)?;
        if validator.delegator_shares.is_zero() && validator.status == BondStatus::Unbonded {
            // if not unbonded, we must instead remove validator in EndBlocker once it finishes its unbonding period
            self.remove_validator(ctx, &validator)?;
        }
        Ok(tokens_amount)
    }

    pub fn unbonded_to_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Unbonded {
            return Err(AppError::Custom(format!(
                "bad state transition unbonded to bonded, validator: {}",
                validator.operator_address
            ))
            .into());
        }
        self.bond_validator(ctx, validator)?;
        Ok(())
    }

    /// ValidateUnbondAmount validates that a given unbond or redelegation amount is
    /// valied based on upon the converted shares. If the amount is valid, the total
    /// amount of respective shares is returned, otherwise an error is returned.
    pub fn validate_unbond_amount<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: &AccAddress,
        val_addr: &ValAddress,
        amount: Uint256,
    ) -> anyhow::Result<Decimal256> {
        let validator = self
            .validator(ctx, val_addr)?
            .ok_or(AppError::AccountNotFound)?;
        let delegation = self
            .delegation(ctx, del_addr, val_addr)?
            .ok_or(AppError::Custom("Delegation is not found.".to_string()))?;
        let mut shares = validator.shares_from_tokens(amount)?;
        let truncated_shares = validator.shares_from_tokens_truncated(amount)?;
        let delegation_shares = delegation.shares;

        if truncated_shares > delegation_shares {
            return Err(AppError::Custom("invalid shares amount".to_string()).into());
        }

        // Cap the shares at the delegation's shares. Shares being greater could occur
        // due to rounding, however we don't want to truncate the shares or take the
        // minimum because we want to allow for the full withdraw of shares from a
        // delegation.
        if shares > delegation_shares {
            shares = delegation_shares;
        }

        Ok(shares)
    }
}
