pub mod client_params;
use axum::{body::Body, routing::get, Router};
use gears::{application::ApplicationInfo, baseapp::{ABCIHandler, Genesis}, client::rest::RestState, x::params::ParamsSubspaceKey};
use proto_messages::cosmos::tx::v1beta1::message::Message;
use store::StoreKey;

use crate::client::cli::query::client_params::CLIENT_PARAMS_URL;

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>() -> Router<RestState<SK, PSK, M, H, G, AI>, Body> {
    Router::new()
        // .route(CLIENT_PARAMS_URL,  get( client_params::client_params ))
        // .route("/v1beta1/balances/:address",  todo!())
}