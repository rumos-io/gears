use axum::{
    extract::{Path, Query, State},
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
        protobuf::Protobuf,
        query::response::QueryConsensusStatesResponse,
        types::core::{
            client::context::types::proto::v1::QueryConsensusStatesRequest,
            host::identifiers::ClientId,
        },
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;
use tendermint::abci::Application;
use tendermint::proto::abci::RequestQuery;

use crate::client::cli::query::consensus_states::CONSENSUS_STATES_URL;

async fn handle<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    Path(client_id): Path<ClientId>,
    Query(_pagination): Query<Option<PageRequest>>,
    State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryConsensusStatesResponse>, Error> {
    let query = QueryConsensusStatesRequest {
        client_id: client_id.to_string(),
        pagination: None,
    };

    let request = RequestQuery {
        data: ProstMessage::encode_to_vec(&query).into(),
        path: CONSENSUS_STATES_URL.to_owned(),
        height: 0,
        prove: false,
    };

    let response = app.query(request);

    Ok(Json(
        QueryConsensusStatesResponse::decode(response.value).map_err(|_| {
            Error::bad_gateway_with_msg("should be a valid QueryConsensusStateResponse".to_owned())
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
    Router::new().route(
        constcat::concat!(CONSENSUS_STATES_URL, "/:client_id"),
        get(handle),
    )
}
