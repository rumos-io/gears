mod client_params;
mod client_state;
mod client_states;
mod client_status;
mod consensus_heights;
mod consensus_state;
mod consensus_states;

use axum::Router;
use gears::{
    application::ApplicationInfo,
    baseapp::{ABCIHandler, Genesis},
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
    AI: ApplicationInfo,
>() -> Router<RestState<SK, PSK, M, H, G, AI>> {
    Router::new()
        .merge(client_params::Route::router())
        .merge(client_state::Route::router())
        .merge(client_states::Route::router())
        .merge(client_status::Route::router())
        .merge(consensus_heights::Route::router())
        .merge(consensus_state::Route::router())
        .merge(consensus_states::Route::router())
}
