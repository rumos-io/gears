use axum::{body::Body, http::Method, routing::get, Router};
use proto_messages::cosmos::tx::v1beta1::Message;

use serde::de::DeserializeOwned;
use std::{hash::Hash, net::SocketAddr};
use store_crate::StoreKey;
use strum::IntoEnumIterator;
use tokio::runtime::Runtime;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    baseapp::{
        ante::{AuthKeeper, BankKeeper},
        BaseApp, Handler,
    },
    client::rest::handlers::{node_info, staking_params},
    x::params::ParamsSubspaceKey,
};

pub fn run_rest_server<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    app: BaseApp<SK, PSK, M, BK, AK, H, G>,
    port: u16,
    router: Router<BaseApp<SK, PSK, M, BK, AK, H, G>, Body>,
) {
    std::thread::spawn(move || {
        Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(launch(app, port, router));
    });
}

// TODO:
// 1. Replace "accept" header to force rocket to return json errors rather than the default HTML.
// 2. what happens if a route panics?
// 3. No error message unrecognized route - does return a 404 - can use a "fallback" route if necessary
async fn launch<
    SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
    PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK, G> + 'static,
    G: DeserializeOwned + Clone + Send + Sync + 'static,
>(
    app: BaseApp<SK, PSK, M, BK, AK, H, G>,
    port: u16,
    router: Router<BaseApp<SK, PSK, M, BK, AK, H, G>, Body>,
) {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/cosmos/base/tendermint/v1beta1/node_info", get(node_info))
        .route("/cosmos/staking/v1beta1/params", get(staking_params))
        .nest("/cosmos", router)
        .layer(cors)
        .with_state(app);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
