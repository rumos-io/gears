use axum::{extract::State, routing::get, Json, Router};
use gears::{
    application::ApplicationInfo,
    baseapp::{ABCIHandler, BaseApp, Genesis},
    client::rest::RestState,
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::{
    ibc::{
        query::QueryClientParamsResponse,
        types::core::client::context::types::proto::v1::QueryClientParamsRequest,
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;

use gears::client::rest::error::Error;

use crate::client::cli::query::client_params::PARAMS_URL;

async fn handle<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    State(_app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryClientParamsResponse>, Error> {
    let _req = QueryClientParamsRequest {};

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
    Router::new().route(PARAMS_URL, get(handle))
}
