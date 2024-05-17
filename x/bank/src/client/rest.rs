use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    baseapp::{genesis::Genesis, BaseApp, NodeQueryHandler, QueryRequest, QueryResponse},
    params::ParamsSubspaceKey,
    rest::{error::Error, Pagination, RestState},
    tendermint::types::{proto::Protobuf, request::query::RequestQuery},
    types::{denom::Denom, tx::TxMessage},
};
use gears::{error::IBC_ENCODE_UNWRAP, store::StoreKey};
use gears::{tendermint::application::ABCIApplication, types::address::AccAddress};
use serde::Deserialize;

use crate::{
    types::query::{
        QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest,
        QueryBalanceResponse, QueryTotalSupplyResponse,
    },
    BankNodeQueryRequest, BankNodeQueryResponse,
};

/// Gets the total supply of every denom
pub async fn supply<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq: QueryRequest,
    QRes: QueryResponse,
>(
    State(app): State<BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>>,
) -> Result<Json<QueryTotalSupplyResponse>, Error> {
    let request = RequestQuery {
        data: Default::default(),
        path: "/cosmos.bank.v1beta1.Query/TotalSupply".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryTotalSupplyResponse::decode(response.value)
            .expect("should be a valid QueryTotalSupplyResponse"),
    ))
}

/// Get all balances for a given address
pub async fn get_balances<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq: QueryRequest,
    QRes: QueryResponse,
>(
    Path(address): Path<AccAddress>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>>,
) -> Result<Json<QueryAllBalancesResponse>, Error> {
    let req = QueryAllBalancesRequest {
        address,
        pagination: None,
    };

    let request = RequestQuery {
        data: req.encode_vec().expect(IBC_ENCODE_UNWRAP).into(), // TODO:IBC
        path: "/cosmos.bank.v1beta1.Query/AllBalances".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryAllBalancesResponse::decode(response.value)
            .expect("should be a valid QueryAllBalancesResponse"),
    ))
}

#[derive(Deserialize)]
pub struct QueryData {
    denom: Denom,
}

// TODO: returns {"balance":null} if balance is zero, is this expected?
/// Get balance for a given address and denom
//#[get("/cosmos/bank/v1beta1/balances/<addr>/by_denom?<denom>")]
pub async fn get_balances_by_denom<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq: QueryRequest + From<BankNodeQueryRequest>,
    QRes: QueryResponse + TryInto<BankNodeQueryResponse>,
>(
    Path(address): Path<AccAddress>,
    query: Query<QueryData>,
    State(app): State<BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>>,
) -> Result<Json<QRes>, Error> {
    let req = BankNodeQueryRequest::Balance(QueryBalanceRequest {
        address,
        denom: query.0.denom,
    });
    let res = app
        .typed_query(req)
        .map_err(|_| Error::internal_server_error())?;

    Ok(Json(res))
}

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq: QueryRequest + From<BankNodeQueryRequest>,
    QRes: QueryResponse + TryInto<BankNodeQueryResponse>,
>() -> Router<RestState<SK, PSK, M, H, G, AI, QReq, QRes>> {
    Router::new()
        .route("/v1beta1/supply", get(supply))
        .route("/v1beta1/balances/:address", get(get_balances))
        .route(
            "/v1beta1/balances/:address/by_denom",
            get(get_balances_by_denom),
        )
}
