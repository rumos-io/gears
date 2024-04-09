use axum::Router;
use gears::application::handlers::ABCIHandler;
use gears::types::tx::TxMessage;
use gears::{
    application::ApplicationInfo, baseapp::Genesis, client::rest::RestState,
    x::params::ParamsSubspaceKey,
};
// use proto_messages::cosmos::tx::v1beta1::message::Message;
use gears::store::StoreKey;

pub fn get_router<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
>() -> Router<RestState<SK, PSK, M, H, G, AI>> {
    Router::new().nest("/cosmos/bank", bank::rest::get_router())
}
