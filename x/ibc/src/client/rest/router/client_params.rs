use axum::{extract::State, routing::get, Json, Router};
use gears::{
    application::ApplicationInfo,
    baseapp::{ABCIHandler, BaseApp, Genesis},
    client::rest::RestState,
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::{
    ibc::{
        protobuf::Protobuf, query::response::QueryClientParamsResponse,
        types::core::client::context::types::proto::v1::QueryClientParamsRequest,
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;

use gears::client::rest::error::Error;
use prost::Message as ProstMessage;
use tendermint::abci::Application;
use tendermint::proto::abci::RequestQuery;

use crate::client::cli::query::client_params::PARAMS_URL;

async fn handle<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryClientParamsResponse>, Error> {
    let query = QueryClientParamsRequest {};

    let request = RequestQuery {
        data: ProstMessage::encode_to_vec(&query).into(),
        path: PARAMS_URL.to_owned(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryClientParamsResponse::decode(response.value).map_err(|_| {
            Error::bad_gateway_with_msg("should be a valid QueryTotalSupplyResponse".to_owned())
        })?,
    ))
}

pub fn router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>() -> Router<RestState<SK, PSK, M, H, G, AI>> {
    Router::new().route(PARAMS_URL, get(handle))
}
