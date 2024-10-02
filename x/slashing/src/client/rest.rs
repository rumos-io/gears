use axum::{extract::State, routing::get, Json, Router};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
};

use crate::{
    QueryParamsRequest, QueryParamsResponse, QuerySigningInfosRequest, SlashingNodeQueryRequest,
    SlashingNodeQueryResponse, SlashingParams,
};

pub async fn signing_infos<
    QReq: QueryRequest + From<SlashingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<SlashingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Query(pagination): Query<Pagination>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = SlashingNodeQueryRequest::SigningInfos(QuerySigningInfosRequest {
        pagination: Some(PaginationRequest::from(pagination)),
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn params<
    QReq: QueryRequest + From<SlashingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<SlashingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = SlashingNodeQueryRequest::Params(QueryParamsRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub async fn const_params() -> Result<Json<QueryParamsResponse>, HTTPError> {
    let res = QueryParamsResponse {
        params: SlashingParams::default(),
    };
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<SlashingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<SlashingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .route("/v1beta1/signing_infos", get(signing_infos))
        .route(
            "/v1beta1/params/current", /* "/v1beta1/params" */
            get(params),
        )
        // TODO: remove const handler and route after integration and update route
        .route("/v1beta1/params", get(const_params))
}
