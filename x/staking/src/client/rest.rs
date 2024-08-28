use crate::{QueryPoolRequest, StakingNodeQueryRequest, StakingNodeQueryResponse};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, Pagination, RestState},
    types::pagination::request::PaginationRequest,
    x::types::validator::BondStatus,
};

pub async fn pool<
    QReq: QueryRequest + From<StakingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<StakingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = StakingNodeQueryRequest::Pool(QueryPoolRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn validators<
    QReq: QueryRequest + From<StakingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<StakingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    // TODO: add status
    pagination: Query<Pagination>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = StakingNodeQueryRequest::Validators(crate::QueryValidatorsRequest {
        // TODO
        status: BondStatus::Unspecified,
        pagination: Some(PaginationRequest::from(pagination.0)),
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<StakingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<StakingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .route("/v1beta1/validators", get(validators))
        .route("/v1beta1/pool", get(pool))
}
