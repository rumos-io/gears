use crate::{DelegatorStartingInfo, SlashEventIterator, ValidatorOutstandingRewards};

use super::*;
use gears::{
    context::QueryableContext,
    error::AppError,
    tendermint::types::proto::event::{Event, EventAttribute},
    types::{
        base::coins::DecimalCoins,
        decimal256::{Decimal256, ONE_DEC, SMALLEST_DEC},
    },
    x::types::delegation::StakingDelegation,
};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        DSK: DistributionStakingKeeper<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, DSK, M>
{
    /// initialize starting info for a new delegation
    pub fn initialize_delegation<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        validator_address: &ValAddress,
        delegator_address: &AccAddress,
    ) -> Result<(), AppError> {
        // period has already been incremented - we want to store the period ended by this delegation action
        let previous_period = self
            .validator_current_rewards(ctx, validator_address)?
            .ok_or(AppError::Custom(
                "validator current rewards are not found".to_string(),
            ))?
            .period
            - 1;
        // increment reference count for the period we're going to track
        self.increment_reference_count(ctx, validator_address, previous_period)?;

        let validator = self
            .staking_keeper
            .validator(ctx, validator_address)?
            .ok_or(AppError::Custom("validator is not found".to_string()))?;
        let delegation = self
            .staking_keeper
            .delegation(ctx, delegator_address, validator_address)?
            .ok_or(AppError::Custom("delegation is not found".to_string()))?;

        // calculate delegation stake in tokens
        // we don't store directly, so multiply delegation shares * (tokens per share)
        // note: necessary to truncate so we don't allow withdrawing more rewards than owed
        let stake = validator.tokens_from_shares(*delegation.shares())?.floor();
        Ok(self.set_delegator_starting_info(
            ctx,
            validator_address,
            delegator_address,
            &DelegatorStartingInfo {
                previous_period,
                stake,
                height: ctx.height() as u64,
            },
        )?)
    }

    pub fn delegation_withdraw_rewards<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        validator: impl StakingValidator,
        delegation: impl StakingDelegation,
    ) -> Result<Option<UnsignedCoins>, AppError> {
        // check existence of delegator starting info
        if !self.has_delegator_starting_info(ctx, delegation.validator(), delegation.delegator())? {
            return Err(AppError::Custom("delegation is not found".to_string()));
        }
        // end current period and calculate rewards
        let ending_period =
            self.increment_validator_period(ctx, validator.operator(), validator.tokens())?;
        let rewards_raw = self
            .calculate_delegation_rewards(
                ctx,
                validator.operator(),
                delegation.delegator(),
                validator.tokens_from_shares(*delegation.shares())?,
                ending_period,
            )?
            .ok_or(AppError::Custom("Can't get delegation rewards".to_string()))?;
        let outstanding = self
            .validator_outstanding_rewards(ctx, delegation.validator())?
            .ok_or(AppError::Custom(
                "cant find validator outstanding rewards".to_string(),
            ))?
            .rewards;

        // defensive edge case may happen on the very final digits
        // of the DecimalCoins due to operation order of the distribution mechanism.
        let rewards = rewards_raw.intersect(&outstanding);

        if rewards.ne(&rewards_raw) {
            tracing::info!(
                name: "rounding error withdrawing rewards from validator",
                target: "module::distribution",
                delegator = delegation.delegator().to_string(),
                validator = validator.operator().to_string(),
                // TODO: implement
                // got = rewards.to_string(),
                // expected = rewards_raw.to_string()
            );
        }

        // truncate reward dec coins, return remainder to community pool
        let (final_rewards, remainder) = rewards.truncate_decimal();

        // add coins to user account
        if final_rewards.is_some() {
            let withdraw_address = self
                .delegator_withdraw_addr(ctx, delegation.delegator())?
                .ok_or(AppError::AccountNotFound)?;
            self.bank_keeper.send_coins_from_module_to_account(
                ctx,
                &withdraw_address,
                &self.distribution_module,
                final_rewards
                    .clone()
                    .expect("expect after check of existence cannot fail"),
            )?;
        }

        // update the outstanding rewards and the community pool only if the
        // transaction was successful
        self.set_validator_outstanding_rewards(
            ctx,
            delegation.validator(),
            &ValidatorOutstandingRewards {
                rewards: outstanding
                    .checked_sub(&rewards)
                    .map_err(|e| AppError::Coins(e.to_string()))?,
            },
        )?;
        let mut fee_pool = self
            .fee_pool(ctx)?
            .ok_or(AppError::Custom("fee pool is not set".to_string()))?;
        if let Some(rem) = remainder {
            fee_pool.community_pool = fee_pool
                .community_pool
                .checked_add(&rem)
                .map_err(|e| AppError::Coins(e.to_string()))?;
            self.set_fee_pool(ctx, &fee_pool)?;
        }

        // decrement reference count of starting period
        let starting_info = self
            .delegator_starting_info(ctx, delegation.validator(), delegation.delegator())?
            .ok_or(AppError::Custom(
                "delegator starting info is not found".to_string(),
            ))?;
        let starting_period = starting_info.previous_period;
        self.decrement_reference_count(ctx, delegation.validator(), starting_period)?;

        // remove delegator starting info
        self.delete_delegator_starting_info(ctx, delegation.validator(), delegation.delegator())?;

        // TODO: do we need this branch?
        // We cannot create invalid coins struct with zero amount coin so it seems like we can skip
        // it and return optional value. Otherwise, consider to implement some restricted struct
        // ZeroCoins and some enum for combining different coins types
        // if final_rewards.is_none() {
        //     baseDenom, _ := sdk.GetBaseDenom()
        //     if baseDenom == "" {
        //         baseDenom = sdk.DefaultBondDenom
        //     }
        //
        //     // Note, we do not call the NewCoins constructor as we do not want the zero
        //     // coin removed.
        //     finalRewards = sdk.Coins{sdk.NewCoin(baseDenom, sdk.ZeroInt())}
        // }

        ctx.push_event(Event {
            r#type: "withdraw_rewards".to_string(),
            attributes: vec![
                EventAttribute {
                    key: "amount".into(),
                    // TODO: stringify coins structs
                    value: serde_json::to_string(&final_rewards).unwrap().into(),
                    index: false,
                },
                EventAttribute {
                    key: "validator".into(),
                    value: validator.operator().to_string().into(),
                    index: false,
                },
            ],
        });

        Ok(final_rewards)
    }

    /// calculate the total rewards accrued by a delegation
    pub fn calculate_delegation_rewards<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        validator_address: &ValAddress,
        delegator_address: &AccAddress,
        tokens: Decimal256,
        ending_period: u64,
    ) -> Result<Option<DecimalCoins>, AppError> {
        // fetch starting info for delegation
        let starting_info = self
            .delegator_starting_info(ctx, validator_address, delegator_address)?
            .ok_or(AppError::Custom(
                "delegation starting info is not found".to_string(),
            ))?;

        if starting_info.height == ctx.height() as u64 {
            // started this height, no rewards yet
            return Ok(None);
        }

        let mut starting_period = starting_info.previous_period;
        let mut stake = starting_info.stake;

        // Iterate through slashes and withdraw with calculated staking for
        // distribution periods. These period offsets are dependent on *when* slashes
        // happen - namely, in begin_block, after rewards are allocated...
        // Slashes which happened in the first block would have been before this
        // delegation existed, UNLESS they were slashes of a redelegation to this
        // validator which was itself slashed (from a fault committed by the
        // redelegation source validator) earlier in the same begin_block.
        let starting_height = starting_info.height;

        // Slashes this block happened after reward allocation, but we have to account
        // for them for the stake sanity check below.
        let ending_height = ctx.height() as u64;

        let mut rewards: Option<DecimalCoins> = None;
        if ending_height > starting_height {
            let slash_event_iterator = SlashEventIterator::new(
                ctx,
                &self.store_key,
                validator_address,
                starting_height,
                ending_height,
            );

            for r in slash_event_iterator {
                let (_, event) = r?;

                let ending_period = event.validator_period;
                if ending_period > starting_period {
                    let addition = self.calculate_delegation_rewards_between(
                        ctx,
                        validator_address,
                        starting_period,
                        ending_period,
                        stake,
                    )?;
                    rewards = if let Some(r) = rewards {
                        Some(
                            r.checked_add(&addition)
                                .map_err(|e| AppError::Coins(e.to_string()))?,
                        )
                    } else {
                        Some(addition)
                    };
                    // Note: It is necessary to truncate so we don't allow withdrawing
                    // more rewards than owed.
                    stake = stake
                        .checked_mul(
                            ONE_DEC
                                .checked_sub(event.fraction)
                                .map_err(|e| AppError::Custom(e.to_string()))?,
                        )
                        .map_err(|e| AppError::Custom(e.to_string()))?
                        .floor();
                    starting_period = ending_period;
                }
            }
        }
        // A total stake sanity check; Recalculated final stake should be less than or
        // equal to current stake here. We cannot use Equals because stake is truncated
        // when multiplied by slash fractions (see above). We could only use equals if
        // we had arbitrary-precision rationals.
        let current_stake = tokens;

        if stake > current_stake {
            // For rounding inconsistencies between:
            //
            //     current_stake: calculated as in staking with a single computation
            //     stake:        calculated as an accumulation of stake
            //                   calculations across validator's distribution periods
            //
            // These inconsistencies are due to differing order of operations which
            // will inevitably have different accumulated rounding and may lead to
            // the smallest decimal place being one greater in stake than
            // current_stake. When we calculated slashing by period, even if we
            // round down for each slash fraction, it's possible due to how much is
            // being rounded that we slash less when slashing by period instead of
            // for when we slash without periods. In other words, the single slash,
            // and the slashing by period could both be rounding down but the
            // slashing by period is simply rounding down less, thus making stake >
            // current_stake
            //
            // A small amount of this error is tolerated and corrected for,
            // however any greater amount should be considered a breach in expected
            // behaviour.

            let margin_of_err = SMALLEST_DEC
                .checked_mul(Decimal256::from_atomics(3u64, 0).expect("hardcoded value can't fail"))
                .expect(
                    "smallest decimal is a valid decimal < 1.0. The multiplication cannot fail",
                );
            if stake
                <= (current_stake
                    .checked_add(margin_of_err)
                    .map_err(|e| AppError::Custom(e.to_string()))?)
            {
                stake = current_stake;
            } else {
                // TODO: seems like panic is a valid exit here. Please, check the legality
                panic!(
                    "calculated final stake for delegator {} greater than current stake
                            \tfinal stake:\t{}
                            \tcurrent stake:\t{}",
                    delegator_address, stake, current_stake
                );
            }
        }

        // calculate rewards for final period

        let addition = self.calculate_delegation_rewards_between(
            ctx,
            validator_address,
            starting_period,
            ending_period,
            stake,
        )?;
        rewards = if let Some(r) = rewards {
            Some(
                r.checked_add(&addition)
                    .map_err(|e| AppError::Coins(e.to_string()))?,
            )
        } else {
            Some(addition)
        };
        Ok(rewards)
    }

    /// calculate the rewards accrued by a delegation between two periods
    pub fn calculate_delegation_rewards_between<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        validator_address: &ValAddress,
        starting_period: u64,
        ending_period: u64,
        stake: Decimal256,
    ) -> Result<DecimalCoins, AppError> {
        // sanity check
        if starting_period > ending_period {
            panic!("starting_period cannot be greater than ending_period");
        }

        // return staking * (ending - starting)
        let starting = self
            .validator_historical_rewards(ctx, validator_address, starting_period)?
            .ok_or(AppError::Custom(
                "cant find validator historical info".to_string(),
            ))?;
        let ending = self
            .validator_historical_rewards(ctx, validator_address, ending_period)?
            .ok_or(AppError::Custom(
                "cant find validator historical info".to_string(),
            ))?;
        // TODO: panics if there are some negative values
        let difference = ending
            .cumulative_reward_ratio
            .checked_sub(&starting.cumulative_reward_ratio)
            .map_err(|e| AppError::Coins(e.to_string()))?;

        // note: necessary to truncate so we don't allow withdrawing more rewards than owed
        difference
            .checked_mul_dec_truncate(stake)
            .map_err(|e| AppError::Coins(e.to_string()))
    }
}
