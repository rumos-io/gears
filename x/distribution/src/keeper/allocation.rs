use super::*;
use crate::{ValidatorAccumulatedCommission, ValidatorCurrentRewards, ValidatorOutstandingRewards};
use gears::{
    context::block::BlockContext,
    error::AppError,
    tendermint::types::proto::{
        event::{Event, EventAttribute},
        info::VoteInfo,
    },
    types::{
        address::ValAddress,
        base::coins::{DecimalCoins, UnsignedCoins},
        decimal256::{Decimal256, ONE_DEC},
    },
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
    /// allocate_tokens handles distribution of the collected fees
    /// bonded_votes is a list of (validator address, validator voted on last block flag) for all
    /// validators in the bonded set.
    pub fn allocate_tokens<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        sum_previous_precommit_power: u64,
        total_previous_power: u64,
        previous_proposer: &ConsAddress,
        bonded_votes: &[VoteInfo],
    ) -> anyhow::Result<()> {
        // fetch and clear the collected fees for distribution, since this is
        // called in begin_block, collected fees will be from the previous block
        // (and distributed to the previous proposer)
        let fees_collected_int = self
            .bank_keeper
            .get_all_balances(ctx, self.fee_collector_module.get_address())
            .unwrap_gas();
        let fees_collected = DecimalCoins::try_from(fees_collected_int.clone())?;

        // transfer collected fees to the distribution module account
        self.bank_keeper.send_coins_from_module_to_module(
            ctx,
            &self.fee_collector_module,
            &self.distribution_module,
            UnsignedCoins::new(fees_collected_int)?,
        )?;

        // temporary workaround to keep CanWithdrawInvariant happy
        // general discussions here: https://github.com/cosmos/cosmos-sdk/issues/2906#issuecomment-441867634
        let mut fee_pool = self.fee_pool(ctx).unwrap_gas().ok_or(AppError::Custom(
            "Stored fee pool should not have been none".to_string(),
        ))?;
        if total_previous_power == 0 {
            fee_pool.community_pool = fee_pool.community_pool.checked_add(&fees_collected)?;
            self.set_fee_pool(ctx, &fee_pool).unwrap_gas();
            return Ok(());
        }

        // calculate fraction votes
        // TODO: reduce conversions
        let previous_fraction_votes = Decimal256::from_atomics(sum_previous_precommit_power, 0)?
            .checked_div(Decimal256::from_atomics(total_previous_power, 0)?)?;

        let params = self.params_keeper.get(ctx);
        // calculate previous proposer reward
        let base_proposer_reward = params.base_proposer_reward;
        let bonus_proposer_reward = params.bonus_proposer_reward;
        let proposer_multiplier = base_proposer_reward.checked_add(
            bonus_proposer_reward
                .checked_mul(previous_fraction_votes)?
                // TODO: check the operation
                .floor(),
        )?;
        let proposer_reward = fees_collected.checked_mul_dec_truncate(proposer_multiplier)?;

        // pay previous proposer
        let mut remaining = if let Some(proposer_validator) = self
            .staking_keeper
            .validator_by_cons_addr(ctx, previous_proposer)
            .unwrap_gas()
        {
            ctx.push_event(Event {
                r#type: "proposer_reward".to_string(),
                attributes: vec![
                    EventAttribute {
                        key: "amount".into(),
                        // TODO: stringify coins structs
                        value: serde_json::to_string(&proposer_reward).unwrap().into(),
                        index: false,
                    },
                    EventAttribute {
                        key: "validator".into(),
                        value: proposer_validator.operator().to_string().into(),
                        index: false,
                    },
                ],
            });

            self.allocate_tokens_to_validator(
                ctx,
                proposer_validator.operator(),
                proposer_validator.commission(),
                &proposer_reward,
            )?;
            fees_collected.checked_sub(&proposer_reward)?
        } else {
            // previous proposer can be unknown if say, the unbonding period is 1 block, so
            // e.g. a validator undelegates at block X, it's removed entirely by
            // block X+1's end_block, then X+2 we need to refer to the previous
            // proposer for X+1, but we've forgotten about them.
            let error = format!("WARNING: Attempt to allocate proposer rewards to unknown proposer {}.
                    This should happen only if the proposer unbonded completely within a single block,
                    which generally should not happen except in exceptional circumstances (or fuzz testing).
                    We recommend you investigate immediately.",
                previous_proposer);
            tracing::error!(error);
            fees_collected.clone()
        };

        // calculate fraction allocated to validators
        let community_tax = params.community_tax;
        let vote_multiplier = ONE_DEC
            .checked_sub(proposer_multiplier)?
            .checked_sub(community_tax)?;
        let fee_multiplier = fees_collected.checked_mul_dec_truncate(vote_multiplier)?;

        // allocate tokens proportionally to voting power
        //
        // TODO: Consider parallelizing later
        // Ref: https://github.com/cosmos/cosmos-sdk/pull/3099#discussion_r246276376
        for vote in bonded_votes {
            let validator = self
                .staking_keeper
                .validator_by_cons_addr(ctx, &ConsAddress::from(vote.validator.address.clone()))
                .unwrap_gas()
                .ok_or(AppError::AccountNotFound)?;
            // TODO: Consider micro-slashing for missing votes.
            //
            // Ref: https://github.com/cosmos/cosmos-sdk/issues/2525#issuecomment-430838701

            let power_fraction = Decimal256::from_atomics(u64::from(vote.validator.power), 0)?
                .checked_div(Decimal256::from_atomics(total_previous_power, 0)?)?
                .floor();
            let reward = fee_multiplier.checked_mul_dec_truncate(power_fraction)?;
            self.allocate_tokens_to_validator(
                ctx,
                validator.operator(),
                validator.commission(),
                &reward,
            )?;
            remaining = remaining.checked_sub(&reward)?;
        }

        // allocate community funding
        fee_pool.community_pool = fee_pool.community_pool.checked_add(&remaining)?;
        self.set_fee_pool(ctx, &fee_pool).unwrap_gas();

        Ok(())
    }

    /// allocate_tokens_to_validator allocate tokens to a particular validator,
    /// splitting according to commission.
    pub fn allocate_tokens_to_validator<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        validator_operator_addr: &ValAddress,
        validator_commission_rate: Decimal256,
        tokens: &DecimalCoins,
    ) -> anyhow::Result<()> {
        // split tokens between validator and delegators according to commission
        let commission = tokens.checked_mul_dec(validator_commission_rate)?;
        let shared = tokens.checked_sub(&commission)?;

        // update current commission
        ctx.push_event(Event {
            r#type: "commission".to_string(),
            attributes: vec![
                EventAttribute {
                    key: "amount".into(),
                    // TODO: stringify coins structs
                    value: serde_json::to_string(&commission).unwrap().into(),
                    index: false,
                },
                EventAttribute {
                    key: "validator".into(),
                    value: validator_operator_addr.to_string().into(),
                    index: false,
                },
            ],
        });

        let current_commission = if let Some(mut current_commission) = self
            .validator_accumulated_commission(ctx, validator_operator_addr)
            .unwrap_gas()
        {
            current_commission.commission =
                current_commission.commission.checked_add(&commission)?;
            current_commission
        } else {
            ValidatorAccumulatedCommission { commission }
        };
        self.set_validator_accumulated_commission(
            ctx,
            validator_operator_addr,
            &current_commission,
        )
        .unwrap_gas();

        // update current rewards
        let current_rewards = if let Some(mut cur_reward) = self
            .validator_current_rewards(ctx, validator_operator_addr)
            .unwrap_gas()
        {
            cur_reward.rewards = cur_reward.rewards.checked_add(&shared)?;
            cur_reward
        } else {
            // TODO: sdk doesn't have this branch
            ValidatorCurrentRewards {
                rewards: shared,
                period: 0,
            }
        };
        self.set_validator_current_rewards(ctx, validator_operator_addr, &current_rewards)
            .unwrap_gas();

        // update outstanding rewards
        ctx.push_event(Event {
            r#type: "rewards".to_string(),
            attributes: vec![
                EventAttribute {
                    key: "amount".into(),
                    // TODO: stringify coins structs
                    value: serde_json::to_string(&tokens).unwrap().into(),
                    index: false,
                },
                EventAttribute {
                    key: "validator".into(),
                    value: validator_operator_addr.to_string().into(),
                    index: false,
                },
            ],
        });

        let outstanding = if let Some(mut outstanding_rewards) = self
            .validator_outstanding_rewards(ctx, validator_operator_addr)
            .unwrap_gas()
        {
            outstanding_rewards.rewards = outstanding_rewards.rewards.checked_add(tokens)?;
            outstanding_rewards
        } else {
            // TODO: sdk doesn't have this branch
            ValidatorOutstandingRewards {
                rewards: tokens.clone(),
            }
        };
        self.set_validator_outstanding_rewards(ctx, validator_operator_addr, &outstanding)
            .unwrap_gas();
        Ok(())
    }
}
