use crate::{ValidatorCurrentRewards, ValidatorHistoricalRewards};

use super::*;
use gears::{
    error::AppError,
    types::{decimal256::Decimal256, uint::Uint256},
};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        SSK: SlashingStakingKeeper<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, SSK, M>
{
    /// increment the reference count for a historical rewards value
    pub fn increment_reference_count<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        validator_address: &ValAddress,
        period: u64,
    ) -> Result<(), AppError> {
        let mut historical = self
            .validator_historical_rewards(ctx, validator_address, period)?
            .ok_or(AppError::Custom(
                "historical rewards are not found".to_string(),
            ))?;
        if historical.reference_count > 2 {
            // TODO: sdk behaviour, seems to be correct
            panic!("reference count should never exceed 2")
        }
        historical.reference_count += 1;
        Ok(self.set_validator_historical_rewards(ctx, validator_address, period, &historical)?)
    }

    /// increment validator period, returning the period just ended
    pub fn increment_validator_period<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        validator_operator_addr: &ValAddress,
        validator_tokens: Uint256,
    ) -> Result<u64, AppError> {
        // fetch current rewards
        let rewards = self
            .validator_current_rewards(ctx, validator_operator_addr)?
            .ok_or(AppError::Custom(
                "validator rewards are not found".to_string(),
            ))?;

        // calculate current ratio
        //     var current sdk.DecCoins
        let current = if validator_tokens.is_zero() {
            // can't calculate ratio for zero-token validators
            // ergo we instead add to the community pool
            let mut fee_pool = self
                .fee_pool(ctx)?
                .ok_or(AppError::Custom("fee pool is not found".to_string()))?;
            let mut outstanding = self
                .validator_outstanding_rewards(ctx, validator_operator_addr)?
                .ok_or(AppError::Custom(
                    "validator outstanding rewards are not found".to_string(),
                ))?;
            fee_pool.community_pool = fee_pool
                .community_pool
                .checked_add(&rewards.rewards)
                .map_err(|e| AppError::Custom(e.to_string()))?;
            outstanding.rewards = outstanding
                .rewards
                .checked_sub(&rewards.rewards)
                .map_err(|e| AppError::Custom(e.to_string()))?;
            self.set_fee_pool(ctx, &fee_pool)?;
            self.set_validator_outstanding_rewards(ctx, validator_operator_addr, &outstanding)?;
            None
        } else {
            // note: necessary to truncate so we don't allow withdrawing more rewards than owed
            Some(
                rewards
                    .rewards
                    .checked_quo_dec_truncate(
                        Decimal256::from_atomics(validator_tokens, 0)
                            .map_err(|e| AppError::Coins(e.to_string()))?,
                    )
                    .map_err(|e| AppError::Coins(e.to_string()))?,
            )
        };

        // fetch historical rewards for last period
        let historical = if let Some(rewards) =
            self.validator_historical_rewards(ctx, validator_operator_addr, rewards.period - 1)?
        {
            rewards.cumulative_reward_ratio
        } else {
            return Err(AppError::Custom(
                "cannot find historical rewards".to_string(),
            ));
        };

        // decrement reference count
        self.decrement_reference_count(ctx, validator_operator_addr, rewards.period - 1)?;

        // set new historical rewards with reference count of 1
        let cumulative_reward_ratio = if let Some(current) = current {
            historical
                .checked_add(&current)
                .map_err(|e| AppError::Coins(e.to_string()))?
        } else {
            historical
        };
        self.set_validator_historical_rewards(
            ctx,
            validator_operator_addr,
            rewards.period,
            &ValidatorHistoricalRewards {
                cumulative_reward_ratio,
                reference_count: 1,
            },
        )?;

        // set current rewards, incrementing period by 1
        self.set_validator_current_rewards(
            ctx,
            validator_operator_addr,
            &ValidatorCurrentRewards {
                // TODO: empty rewards in the sdk
                rewards: rewards.rewards,
                period: rewards.period + 1,
            },
        )?;

        Ok(rewards.period)
    }

    /// decrement the reference count for a historical rewards value, and delete if zero references remain
    pub fn decrement_reference_count<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        validator_operator_addr: &ValAddress,
        period: u64,
    ) -> Result<(), AppError> {
        let mut historical = if let Some(rewards) =
            self.validator_historical_rewards(ctx, validator_operator_addr, period)?
        {
            rewards
        } else {
            return Err(AppError::Custom(
                "cannot find historical rewards".to_string(),
            ));
        };
        if historical.reference_count == 0 {
            // TODO: panics in sdk
            return Err(AppError::Custom(
                "cannot set negative reference count".to_string(),
            ));
        }

        historical.reference_count -= 1;
        if historical.reference_count == 0 {
            self.delete_validator_historical_rewards(ctx, validator_operator_addr, period)?;
        } else {
            self.set_validator_historical_rewards(
                ctx,
                validator_operator_addr,
                period,
                &historical,
            )?;
        }
        Ok(())
    }
}
