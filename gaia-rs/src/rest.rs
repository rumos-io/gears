use axum::{body::Body, Router};
use gears::{
    baseapp::{ante::AnteHandler, ABCIHandler, Genesis},
    client::rest::RestState,
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::tx::v1beta1::Message;
use store::StoreKey;

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandler<SK>,
>() -> Router<RestState<SK, PSK, M, H, G, Ante>, Body> {
    Router::new().nest("/cosmos/bank", bank::rest::get_router())
}
