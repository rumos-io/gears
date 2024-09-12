use axum::{extract::State, routing::get, Json, Router};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
};

use crate::{QueryParamsRequest, SlashingNodeQueryRequest, SlashingNodeQueryResponse};

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

pub async fn const_params() -> &'static str {
    r#"{
      "params": {
        "signed_blocks_window": "10000",
        "min_signed_per_window": "0.050000000000000000",
        "downtime_jail_duration": "600s",
        "slash_fraction_double_sign": "0.050000000000000000",
        "slash_fraction_downtime":"0.000100000000000000"
      }
    }"#
}

pub fn get_router<
    QReq: QueryRequest + From<SlashingNodeQueryRequest>,
    QRes: QueryResponse + TryInto<SlashingNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    // TODO: remove const handler and route after integration and update route
    Router::new()
        .route("/v1beta1/params/current", get(params))
        .route("/v1beta1/params", get(const_params))
}
