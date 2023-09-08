use axum::{body::Body, Router};
use gears::{
    baseapp::{
        ante::{AuthKeeper, BankKeeper},
        Genesis, Handler,
    },
    client::rest::RestState,
    x::params::ParamsSubspaceKey,
};
use proto_messages::cosmos::tx::v1beta1::Message;
use store::StoreKey;

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
>() -> Router<RestState<SK, PSK, M, BK, AK, H, G>, Body> {
    Router::new().nest("/bank", bank::rest::get_router())
}
