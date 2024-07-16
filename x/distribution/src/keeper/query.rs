use gears::context::query::QueryContext;

use crate::{
    QueryParamsRequest, QueryParamsResponse, QueryValidatorOutstandingRewardsRequest,
    QueryValidatorOutstandingRewardsResponse,
};

use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        SSK: SlashingStakingKeeper<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, SSK, M>
{
    pub fn query_params<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        _query: QueryParamsRequest,
    ) -> QueryParamsResponse {
        QueryParamsResponse {
            params: self.params_keeper.get(ctx),
        }
    }

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
}
