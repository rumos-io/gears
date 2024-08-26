use crate::{query::QueryAccountRequest, AuthNodeQueryRequest, AuthNodeQueryResponse};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use gears::types::address::AccAddress;
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
};

/// Get a particular account data.
pub async fn get_account<
    QReq: QueryRequest + From<AuthNodeQueryRequest>,
    QRes: QueryResponse + TryInto<AuthNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Path(address): Path<AccAddress>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = AuthNodeQueryRequest::Account(QueryAccountRequest { address });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<AuthNodeQueryRequest>,
    QRes: QueryResponse + TryInto<AuthNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new().route(
        "/v1beta1/accounts/:address",
        get(get_account::<QReq, QRes, App>),
    )
}
