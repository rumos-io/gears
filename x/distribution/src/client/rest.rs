use crate::{
    DistributionNodeQueryRequest, DistributionNodeQueryResponse, QueryCommunityPoolRequest,
    QueryDelegatorParams, QueryParamsRequest,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
    types::address::AccAddress,
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

pub fn get_router<
    QReq: QueryRequest + From<DistributionNodeQueryRequest>,
    QRes: QueryResponse + TryInto<DistributionNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .route("/v1beta1/community_pool", get(community_pool))
        // TODO: check path
        .route(
            "/v1beta1/delegators/:delegator_address/rewards",
            get(delegation_delegator_rewards),
        )
        .route("/v1beta1/params", get(params))
}
