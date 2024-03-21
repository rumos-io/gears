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
use proto_messages::cosmos::{
    bank::v1beta1::PageRequest,
    ibc::{
        query::QueryClientStateResponse,
        types::core::client::context::types::proto::v1::QueryClientStatesRequest,
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;

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
    State(_app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryClientStateResponse>, Error> {
    let _req = QueryClientStatesRequest { pagination: None };

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
    Router::new().route(STATES_URL, get(handle))
}
