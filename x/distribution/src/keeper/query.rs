use super::*;
use crate::{
    QueryParamsRequest, QueryParamsResponse, QueryValidatorCommissionRequest,
    QueryValidatorCommissionResponse, QueryValidatorOutstandingRewardsRequest,
    QueryValidatorOutstandingRewardsResponse, QueryValidatorSlashesRequest,
    QueryValidatorSlashesResponse, SlashEventIterator,
};
use gears::{context::query::QueryContext, types::pagination::response::PaginationResponse};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        SSK: SlashingStakingKeeper<SK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, SSK, M>
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
        let slash_events_iterator = SlashEventIterator::new(
            ctx,
            &self.store_key,
            &validator_address,
            starting_height,
            ending_height,
        );
        let mut events = vec![];
        for res in slash_events_iterator {
            let (_, event) = res.unwrap_gas();
            events.push(event);
        }
        let total = events.len();

        QueryValidatorSlashesResponse {
            slashes: events,
            pagination: pagination.map(|_| PaginationResponse::new(total)),
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
