use axum::{
    extract::{Path, State},
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
    ibc::{
        query::QueryClientStatusResponse,
        types::core::{
            client::context::types::proto::v1::QueryClientStatusRequest,
            host::identifiers::ClientId,
        },
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;

use crate::client::cli::query::client_status::STATUS_URL;

async fn handle<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    Path(client_id): Path<ClientId>,
    State(_app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryClientStatusResponse>, Error> {
    let _req = QueryClientStatusRequest {
        client_id: client_id.to_string(),
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
    Router::new().route(constcat::concat!(STATUS_URL, "/:client_id"), get(handle))
}
