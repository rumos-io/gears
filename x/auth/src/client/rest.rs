use crate::{
    query::{QueryAccountRequest, QueryAccountsRequest, QueryParamsRequest},
    AuthNodeQueryRequest, AuthNodeQueryResponse,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{NodeQueryHandler, QueryRequest, QueryResponse},
    rest::{error::HTTPError, RestState},
};
use gears::{
    rest::Pagination,
    types::{address::AccAddress, pagination::request::PaginationRequest},
};

/// Get a particular account data.
pub async fn account<
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

/// Get all account data.
pub async fn accounts<
    QReq: QueryRequest + From<AuthNodeQueryRequest>,
    QRes: QueryResponse + TryInto<AuthNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    Query(pagination): Query<Pagination>,
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = AuthNodeQueryRequest::Accounts(QueryAccountsRequest {
        pagination: Some(PaginationRequest::from(pagination)),
    });
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

/// Get module params.
pub async fn params<
    QReq: QueryRequest + From<AuthNodeQueryRequest>,
    QRes: QueryResponse + TryInto<AuthNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>(
    State(rest_state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<QRes>, HTTPError> {
    let req = AuthNodeQueryRequest::Params(QueryParamsRequest {});
    let res = rest_state.app.typed_query(req)?;
    Ok(Json(res))
}

pub fn get_router<
    QReq: QueryRequest + From<AuthNodeQueryRequest>,
    QRes: QueryResponse + TryInto<AuthNodeQueryResponse>,
    App: NodeQueryHandler<QReq, QRes>,
>() -> Router<RestState<QReq, QRes, App>> {
    Router::new()
        .route("/v1beta1/accounts/:address", get(account))
        .route("/v1beta1/accounts", get(accounts))
        .route("/v1beta1/params", get(params))
}
