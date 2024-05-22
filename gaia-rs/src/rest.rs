use axum::Router;
use gears::application::handlers::node::ABCIHandler;
use gears::params_v2::ParamsSubspaceKey;
use gears::store::StoreKey;
use gears::types::tx::TxMessage;
use gears::{application::ApplicationInfo, baseapp::genesis::Genesis, rest::RestState};

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
