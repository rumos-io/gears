use axum::{body::Body, Router};
use gears::{application::ApplicationInfo, baseapp::{ABCIHandler, Genesis}, client::rest::RestState, x::params::ParamsSubspaceKey};
use proto_messages::cosmos::tx::v1beta1::message::Message;
use store::StoreKey;

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>() -> Router<RestState<SK, PSK, M, H, G, AI>, Body> {
    Router::new()
        // .route("/v1beta1/supply",  todo!())
        // .route("/v1beta1/balances/:address",  todo!())
}