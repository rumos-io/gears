use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use gears::{
    application::ApplicationInfo,
    baseapp::{ABCIHandler, BaseApp, Genesis},
    client::rest::{error::Error, RestState},
    x::params::ParamsSubspaceKey,
};
use prost::Message as ProstMessage;
use proto_messages::cosmos::{
    bank::v1beta1::PageRequest,
    ibc::{
        protobuf::Protobuf, query::QueryClientStateResponse,
        types::core::client::context::types::proto::v1::QueryClientStatesRequest,
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;
use tendermint::abci::Application;
use tendermint::proto::abci::RequestQuery;

use crate::client::cli::query::client_states::STATES_URL;

async fn handle<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    Query(_pagination): Query<Option<PageRequest>>,
    State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryClientStateResponse>, Error> {
    let query = QueryClientStatesRequest { pagination: None };

    let request = RequestQuery {
        data: ProstMessage::encode_to_vec(&query).into(),
        path: STATES_URL.to_owned(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryClientStateResponse::decode(response.value).map_err(|_| {
            Error::bad_gateway_with_msg("should be a valid QueryClientStateResponse".to_owned())
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
    Router::new().route(STATES_URL, get(handle))
}
