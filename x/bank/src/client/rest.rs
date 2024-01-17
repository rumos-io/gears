use proto_types::AccAddress;
use tendermint_abci::Application;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    baseapp::{ante::AnteHandlerTrait, BaseApp, Genesis, ABCIHandler},
    client::rest::{error::Error, Pagination, RestState},
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::{
    bank::v1beta1::{
        QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest,
        QueryBalanceResponse, QueryTotalSupplyResponse,
    },
    tx::v1beta1::message::Message, ibc_types::protobuf::Protobuf,
};
use serde::Deserialize;
use store::StoreKey;
use tendermint_proto::abci::RequestQuery;

/// Gets the total supply of every denom
pub async fn supply<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
>(
    State(app): State<BaseApp<SK, PSK, M, H, G, Ante>>,
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
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
>(
    Path(address): Path<AccAddress>,
    _pagination: Query<Pagination>,
    State(app): State<BaseApp<SK, PSK, M, H, G, Ante>>,
) -> Result<Json<QueryAllBalancesResponse>, Error> {
    let req = QueryAllBalancesRequest {
        address,
        pagination: None,
    };

    let request = RequestQuery {
        data: req.encode_vec().into(),
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
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
>(
    Path(address): Path<AccAddress>,
    denom: Query<RawDenom>,
    State(app): State<BaseApp<SK, PSK, M, H, G, Ante>>,
) -> Result<Json<QueryBalanceResponse>, Error> {
    let req = QueryBalanceRequest {
        address,
        denom: denom
            .0
            .denom
            .try_into()
            .map_err(|e: proto_types::Error| Error::bad_request(e.to_string()))?,
    };

    let request: RequestQuery = RequestQuery {
        data: req.encode_vec().into(),
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
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
>() -> Router<RestState<SK, PSK, M, H, G, Ante>, Body> {
    Router::new()
        .route("/v1beta1/supply", get(supply))
        .route("/v1beta1/balances/:address", get(get_balances))
        .route(
            "/v1beta1/balances/:address/by_denom",
            get(get_balances_by_denom),
        )
}
