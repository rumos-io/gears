use super::*;
use crate::{
    DelegationDelegatorReward, QueryCommunityPoolRequest, QueryCommunityPoolResponse,
    QueryDelegationRewardsRequest, QueryDelegationRewardsResponse, QueryDelegatorParams,
    QueryDelegatorTotalRewardsResponse, QueryParamsRequest, QueryParamsResponse,
    QueryValidatorCommissionRequest, QueryValidatorCommissionResponse,
    QueryValidatorOutstandingRewardsRequest, QueryValidatorOutstandingRewardsResponse,
    QueryValidatorSlashesRequest, QueryValidatorSlashesResponse, QueryWithdrawAllRewardsRequest,
    QueryWithdrawAllRewardsResponse, SlashEventIterator,
};
use gears::{
    baseapp::errors::QueryError,
    context::query::QueryContext,
    ext::{IteratorPaginate, Pagination},
    types::pagination::response::PaginationResponse,
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
    pub fn query_validator_outstanding_rewards<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryValidatorOutstandingRewardsRequest,
    ) -> QueryValidatorOutstandingRewardsResponse {
        let rewards = self
            .validator_outstanding_rewards(ctx, &query.validator_address)
            .unwrap_gas();
        QueryValidatorOutstandingRewardsResponse { rewards }
    }

    pub fn query_validator_commission<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryValidatorCommissionRequest,
    ) -> QueryValidatorCommissionResponse {
        let commission = self
            .validator_accumulated_commission(ctx, &query.validator_address)
            .unwrap_gas();
        QueryValidatorCommissionResponse { commission }
    }

    pub fn query_validator_slashes<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryValidatorSlashesRequest {
            validator_address,
            starting_height,
            ending_height,
            pagination,
        }: QueryValidatorSlashesRequest,
    ) -> QueryValidatorSlashesResponse {
        let (pagination_result, slash_events_iterator) = SlashEventIterator::new(
            ctx,
            &self.store_key,
            &validator_address,
            starting_height,
            ending_height,
        )
        .maybe_paginate(pagination.map(Pagination::from));

        let mut events = vec![];
        for res in slash_events_iterator {
            let (_, event) = res.unwrap_gas();
            events.push(event);
        }

        QueryValidatorSlashesResponse {
            slashes: events,
            pagination: pagination_result.map(PaginationResponse::from),
        }
    }

    pub fn query_delegation_rewards<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryDelegationRewardsRequest {
            delegator_address,
            validator_address,
        }: QueryDelegationRewardsRequest,
    ) -> Result<QueryDelegationRewardsResponse, QueryError> {
        // TODO: original logic, can't implement and it's wrong idea to modify state via query
        //       do we have a way to have isolated transactional context that doesn't affect state?
        //     // branch the context to isolate state changes
        //     let validator = self
        //         .staking_keeper
        //         .validator(ctx, &validator_address)
        //         .unwrap_gas()
        //         .ok_or(AppError::AccountNotFound)?;
        //     let delegation = self
        //         .staking_keeper
        //         .delegation(ctx, &delegator_address, &validator_address)
        //         .unwrap_gas()
        //         .ok_or(AppError::Custom("delegation is not found".to_string()))?;
        // let ending_period =
        //     self.increment_validator_period(ctx, validator.operator(), validator.tokens())?;
        // let rewards = self.calculate_delegation_rewards(
        //     ctx,
        //     &validator_address,
        //     &delegator_address,
        //     validator.tokens_from_shares(*delegation.shares())?,
        //     ending_period,
        // )?;

        // logic is checked the ending_period is the period from current rewards
        if let Some(rew) = self
            .validator_current_rewards(ctx, &validator_address)
            .unwrap_gas()
        {
            let validator = self
                .staking_keeper
                .validator(ctx, &validator_address)
                .unwrap_gas()
                .ok_or(QueryError::TODO(anyhow!("account is not found")))?;
            let delegation = self
                .staking_keeper
                .delegation(ctx, &delegator_address, &validator_address)
                .unwrap_gas()
                .ok_or(QueryError::TODO(anyhow!("delegation is not found")))?;
            let rewards = self
                .calculate_delegation_rewards(
                    ctx,
                    &validator_address,
                    &delegator_address,
                    validator
                        .tokens_from_shares(*delegation.shares())
                        .map_err(|e| QueryError::TODO(anyhow!(e.to_string())))?,
                    rew.period,
                )
                .map_err(|e| QueryError::TODO(anyhow!(e.to_string())))?;
            Ok(QueryDelegationRewardsResponse { rewards })
        } else {
            Ok(QueryDelegationRewardsResponse { rewards: None })
        }
    }

    pub fn query_delegator_total_rewards<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryDelegatorParams { delegator_address }: QueryDelegatorParams,
    ) -> QueryDelegatorTotalRewardsResponse {
        let mut del_rewards = vec![];
        for delegation in self
            .staking_keeper
            .delegations_iter(ctx, &delegator_address)
        {
            let delegation = delegation.unwrap_gas();
            let validator_address = delegation.validator();
            if let Some(validator) = self
                .staking_keeper
                .validator(ctx, validator_address)
                .unwrap_gas()
            {
                if let Ok(shares) = validator.tokens_from_shares(*delegation.shares()) {
                    if let Some(rew) = self
                        .validator_current_rewards(ctx, validator_address)
                        .unwrap_gas()
                    {
                        // TODO: original logic, can't implement and it's wrong idea to modify state via query
                        // let ending_period =
                        //     self.increment_validator_period(ctx, validator_address, validator.tokens());
                        let ending_period = rew.period;
                        del_rewards.push(
                            self.calculate_delegation_rewards(
                                ctx,
                                validator_address,
                                &delegator_address,
                                shares,
                                ending_period,
                            )
                            .ok()
                            .flatten()
                            .map(|coins| (coins, validator_address.clone())),
                        );
                    }
                }
            }
        }

        let rewards: Vec<DelegationDelegatorReward> = del_rewards
            .into_iter()
            .flatten()
            .map(|(coins, validator_address)| DelegationDelegatorReward {
                validator_address,
                reward: coins,
            })
            .collect();

        if !rewards.is_empty() {
            let total = rewards[0].reward.clone();
            let total = rewards
                .iter()
                .skip(1)
                .try_fold(total, |acc, v| acc.checked_add(&v.reward))
                .ok();
            QueryDelegatorTotalRewardsResponse { rewards, total }
        } else {
            QueryDelegatorTotalRewardsResponse {
                rewards,
                total: None,
            }
        }
    }

    pub fn query_delegator_validators<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryWithdrawAllRewardsRequest { delegator_address }: QueryWithdrawAllRewardsRequest,
    ) -> QueryWithdrawAllRewardsResponse {
        let validators = self
            .staking_keeper
            .delegations_iter(ctx, &delegator_address)
            .map(|res| {
                let del = res.unwrap_gas();
                del.validator().to_string()
            })
            .collect::<Vec<_>>();
        QueryWithdrawAllRewardsResponse { validators }
    }

    pub fn query_community_pool<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        _query: QueryCommunityPoolRequest,
    ) -> QueryCommunityPoolResponse {
        QueryCommunityPoolResponse {
            pool: self
                .fee_pool(ctx)
                .unwrap_gas()
                .map(|fee_pool| fee_pool.community_pool),
        }
    }

    pub fn query_params<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        _query: QueryParamsRequest,
    ) -> QueryParamsResponse {
        QueryParamsResponse {
            params: self.params_keeper.get(ctx),
        }
    }
}
