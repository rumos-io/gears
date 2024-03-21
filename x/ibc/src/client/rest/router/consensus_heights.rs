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
use proto_messages::cosmos::{
    bank::v1beta1::PageRequest,
    ibc::{
        query::QueryConsensusStateHeightsResponse,
        types::core::{
            client::context::types::proto::v1::QueryConsensusStateHeightsRequest,
            host::identifiers::ClientId,
        },
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;

use crate::client::cli::query::consensus_heights::CONSESUS_HEIGHTS_URL;

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
    State(_app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryConsensusStateHeightsResponse>, Error> {
    let _req = QueryConsensusStateHeightsRequest {
        client_id: client_id.to_string(),
        pagination: None,
    };

    todo!()
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
        constcat::concat!(CONSESUS_HEIGHTS_URL, "/:client_id"),
        get(handle),
    )
}
