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
        query::QueryConsensusStateResponse,
        types::core::{
            client::context::types::proto::v1::QueryConsensusStateRequest,
            host::identifiers::ClientId,
        },
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;

use crate::client::cli::query::consensus_state::CONSENSUS_STATE_URL;

async fn handle<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    Path((client_id, revision_number, revision_height, latest_height)): Path<(
        ClientId,
        u64,
        u64,
        bool,
    )>,
    Query(_pagination): Query<Option<PageRequest>>,
    State(_app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryConsensusStateResponse>, Error> {
    let _req = QueryConsensusStateRequest {
        client_id: client_id.to_string(),
        revision_number,
        revision_height,
        latest_height,
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
    Router::new() // TODO:
        .route(
            constcat::concat!(
                CONSENSUS_STATE_URL,
                "/:client_id/:revision_number/:revision_height/:latest_height"
            ),
            get(handle),
        )
}
