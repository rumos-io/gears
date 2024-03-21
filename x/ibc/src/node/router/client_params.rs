use axum::{extract::State, Json};
use gears::{application::ApplicationInfo, baseapp::{ABCIHandler, BaseApp, Genesis}, x::params::ParamsSubspaceKey};
use proto_messages::cosmos::{ibc::{query::QueryClientParamsResponse, types::core::client::types::Params}, tx::v1beta1::message::Message};
use store::StoreKey;

use gears::client::rest::error::Error;

pub fn client_params<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>(
    State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
) -> Result<Json<QueryClientParamsResponse>, Error> {
    Ok(Json(
        QueryClientParamsResponse{ params: Params { allowed_clients: Default::default() } }
    ))
}