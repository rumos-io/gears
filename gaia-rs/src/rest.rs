use axum::{body::Body, Router};
use gears::{
    baseapp::{ante::AnteHandlerTrait, Genesis, Handler},
    client::rest::RestState,
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::tx::v1beta1::message::Message;
use store::StoreKey;

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
>() -> Router<RestState<SK, PSK, M, H, G, Ante>, Body> {
    Router::new().nest("/cosmos/bank", bank::rest::get_router())
}
