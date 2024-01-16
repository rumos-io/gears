use axum::{body::Body, extract::FromRef, http::Method, routing::get, Router};
use proto_messages::cosmos::tx::v1beta1::message::Message;
use tendermint_rpc::Url;

use std::net::SocketAddr;
use store_crate::StoreKey;
use tokio::runtime::Runtime;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    baseapp::{ante::AnteHandler, ABCIHandler, BaseApp, Genesis},
    client::rest::handlers::{node_info, staking_params, txs},
    x::params::ParamsSubspaceKey,
};

pub fn run_rest_server<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
>(
    app: BaseApp<SK, PSK, M, H, G, Ante>,
    listen_addr: SocketAddr,
    router: Router<RestState<SK, PSK, M, H, G, Ante>, Body>,
    tendermint_rpc_address: Url,
) {
    std::thread::spawn(move || {
        Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(launch(app, listen_addr, router, tendermint_rpc_address));
    });
}

#[derive(Clone)]
pub struct RestState<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
> {
    app: BaseApp<SK, PSK, M, H, G, Ante>,
    tendermint_rpc_address: Url,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: Message,
        H: ABCIHandler<M, SK, G>,
        G: Genesis,
        Ante: AnteHandlerTrait<SK>,
    > FromRef<RestState<SK, PSK, M, H, G, Ante>> for BaseApp<SK, PSK, M, H, G, Ante>
{
    fn from_ref(rest_state: &RestState<SK, PSK, M, H, G, Ante>) -> BaseApp<SK, PSK, M, H, G, Ante> {
        rest_state.app.clone()
    }
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: Message,
        H: ABCIHandler<M, SK, G>,
        G: Genesis,
        Ante: AnteHandlerTrait<SK>,
    > FromRef<RestState<SK, PSK, M, H, G, Ante>> for Url
{
    fn from_ref(rest_state: &RestState<SK, PSK, M, H, G, Ante>) -> Url {
        rest_state.tendermint_rpc_address.clone()
    }
}

// TODO:
// 1. Replace "accept" header to force rocket to return json errors rather than the default HTML.
// 2. what happens if a route panics?
// 3. No error message unrecognized route - does return a 404 - can use a "fallback" route if necessary
async fn launch<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    Ante: AnteHandlerTrait<SK>,
>(
    app: BaseApp<SK, PSK, M, H, G, Ante>,
    listen_addr: SocketAddr,
    router: Router<RestState<SK, PSK, M, H, G, Ante>, Body>,
    tendermint_rpc_address: Url,
) {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let rest_state = RestState {
        app,
        tendermint_rpc_address,
    };

    let app = Router::new()
        .route("/cosmos/base/tendermint/v1beta1/node_info", get(node_info))
        .route("/cosmos/staking/v1beta1/params", get(staking_params))
        .route("/cosmos/tx/v1beta1/txs", get(txs::<M>))
        .merge(router)
        .layer(cors)
        .with_state(rest_state);

    tracing::info!("REST server running at {}", listen_addr);
    axum::Server::bind(&listen_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
