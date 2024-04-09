// use :tendermint::abci::Application;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use gears::store::StoreKey;
use gears::tendermint::application::ABCIApplication;
use gears::{
    application::{handlers::ABCIHandler, ApplicationInfo},
    baseapp::{genesis::Genesis, BaseApp},
    ibc::address::AccAddress,
    rest::{error::Error, Pagination, RestState},
    tendermint::types::{proto::Protobuf, request::query::RequestQuery},
    types::tx::TxMessage,
    x::params::ParamsSubspaceKey,
};
use serde::Deserialize;

use crate::types::query::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryTotalSupplyResponse,
};
// use tendermint::proto::abci::RequestQuery;

/// Gets the total supply of every denom
pub async fn supply<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
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
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    Path(address): Path<AccAddress>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryAllBalancesResponse>, Error> {
    let req = QueryAllBalancesRequest {
        address,
        pagination: None,
    };

    let request = RequestQuery {
        data: req.encode_vec().expect("msg").into(), // TODO:NOW
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
pub struct RawDenom {
    denom: String,
}

// TODO: returns {"balance":null} if balance is zero, is this expected?
/// Get balance for a given address and denom
//#[get("/cosmos/bank/v1beta1/balances/<addr>/by_denom?<denom>")]
pub async fn get_balances_by_denom<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    Path(address): Path<AccAddress>,
    denom: Query<RawDenom>,
    State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryBalanceResponse>, Error> {
    let req = QueryBalanceRequest {
        address,
        denom: denom
            .0
            .denom
            .try_into()
            .map_err(|e: gears::proto_types::error::Error| Error::bad_request(e.to_string()))?,
    };

    let request: RequestQuery = RequestQuery {
        data: req.encode_vec().expect("msg").into(), // TODO:NOW
        path: "/cosmos.bank.v1beta1.Query/Balance".into(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryBalanceResponse::decode(response.value)
            .expect("should be a valid QueryBalanceResponse"),
    ))
}

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>() -> Router<RestState<SK, PSK, M, H, G, AI>> {
    Router::new()
        .route("/v1beta1/supply", get(supply))
        .route("/v1beta1/balances/:address", get(get_balances))
        .route(
            "/v1beta1/balances/:address/by_denom",
            get(get_balances_by_denom),
        )
}
