use crate::{
    DistributionNodeQueryRequest, DistributionNodeQueryResponse, DistributionParams,
    QueryCommunityPoolRequest, QueryCommunityPoolResponse, QueryDelegatorParams,
    QueryParamsRequest, QueryParamsResponse, QueryValidatorCommissionRequest,
    QueryValidatorOutstandingRewardsRequest, QueryValidatorSlashesRequest,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, Pagination, RestState},
    types::{
        address::{AccAddress, ValAddress},
        pagination::request::PaginationRequest,
    },
};

pub async fn delegation_delegator_rewards<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(delegator_address): Path<AccAddress>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::DelegatorTotalRewards(QueryDelegatorParams {
        delegator_address,
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn community_pool<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::CommunityPool(QueryCommunityPoolRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn const_community_pool() -> Result<Json<QueryCommunityPoolResponse>, HTTPError> {
    let res = QueryCommunityPoolResponse {
        pool: gears::types::base::coins::DecimalCoins::new(vec![
            gears::types::base::coin::DecimalCoin {
                denom: "uatom".try_into().expect("hardcoded value cannot fail"),
                amount: gears::types::decimal256::Decimal256::from_atomics(10_000_000_000u64, 1)
                    .expect("Default is valid"),
            },
        ])
        .ok(),
    };
    Ok(Json(res))
}

pub async fn validator_commission<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(validator_address): Path<ValAddress>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::ValidatorCommission(QueryValidatorCommissionRequest {
        validator_address,
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn validator_outstanding_rewards<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(validator_address): Path<ValAddress>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::ValidatorOutstandingRewards(
        QueryValidatorOutstandingRewardsRequest { validator_address },
    );
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn validator_slashes<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(validator_address): Path<ValAddress>,
    Query(pagination): Query<Pagination>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::ValidatorSlashes(QueryValidatorSlashesRequest {
        validator_address,
        // TODO: does it use a query height parameters?
        starting_height: u64::MIN,
        ending_height: u64::MAX,
        pagination: Some(PaginationRequest::from(pagination)),
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn params<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = DistributionNodeQueryRequest::Params(QueryParamsRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn const_params() -> Result<Json<QueryParamsResponse>, HTTPError> {
    let res = QueryParamsResponse {
        params: DistributionParams::default(),
    };
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .route(
            "/v1beta1/community_pool/current", /* "/v1beta1/community_pool" */
            get(community_pool),
        )
        // TODO: remove const handler and route after integration and update route
        .route("/v1beta1/community_pool", get(const_community_pool))
        .route(
            "/v1beta1/delegators/:delegator_address/rewards",
            get(delegation_delegator_rewards),
        )
        .route(
            "/v1beta1/validators/:validator_address/commission",
            get(validator_commission),
        )
        .route(
            "/v1beta1/validators/:validator_address/outstanding_rewards",
            get(validator_outstanding_rewards),
        )
        .route(
            "/v1beta1/validators/:validator_address/slashes",
            get(validator_slashes),
        )
        .route(
            "/v1beta1/params/current", /* "/v1beta1/params" */
            get(params),
        )
        // TODO: remove const handler and route after integration and update route
        .route("/v1beta1/params", get(const_params))
}
