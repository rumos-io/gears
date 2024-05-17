use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    baseapp::{genesis::Genesis, BaseApp, QueryRequest, QueryResponse},
    params::ParamsSubspaceKey,
    rest::handlers::{node_info, staking_params, txs},
    runtime::runtime,
    types::tx::TxMessage,
};
use axum::{extract::FromRef, http::Method, routing::get, Router};
use std::net::SocketAddr;
use store_crate::StoreKey;
use tendermint::rpc::url::Url;
use tower_http::cors::{Any, CorsLayer};

pub fn run_rest_server<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq: QueryRequest,
    QRes: QueryResponse,
>(
    app: BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>,
    listen_addr: SocketAddr,
    router: Router<RestState<SK, PSK, M, H, G, AI, QReq, QRes>>,
    tendermint_rpc_address: Url,
) {
    std::thread::spawn(move || {
        let result = runtime().block_on(launch(app, listen_addr, router, tendermint_rpc_address));
        if let Err(err) = result {
            panic!("Failed to run rest server with err: {}", err)
        }
    });
}

#[derive(Clone)]
pub struct RestState<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq,
    QRes,
> {
    app: BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>,
    tendermint_rpc_address: Url,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: TxMessage,
        H: ABCIHandler<M, SK, G, QReq, QRes>,
        G: Genesis,
        AI: ApplicationInfo,
        QReq: QueryRequest,
        QRes: QueryResponse,
    > FromRef<RestState<SK, PSK, M, H, G, AI, QReq, QRes>>
    for BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>
{
    fn from_ref(
        rest_state: &RestState<SK, PSK, M, H, G, AI, QReq, QRes>,
    ) -> BaseApp<SK, PSK, M, H, G, AI, QReq, QRes> {
        rest_state.app.clone()
    }
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: TxMessage,
        H: ABCIHandler<M, SK, G, QReq, QRes>,
        G: Genesis,
        AI: ApplicationInfo,
        QReq,
        QRes,
    > FromRef<RestState<SK, PSK, M, H, G, AI, QReq, QRes>> for Url
{
    fn from_ref(rest_state: &RestState<SK, PSK, M, H, G, AI, QReq, QRes>) -> Url {
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
    M: TxMessage,
    H: ABCIHandler<M, SK, G, QReq, QRes>,
    G: Genesis,
    AI: ApplicationInfo,
    QReq: QueryRequest,
    QRes: QueryResponse,
>(
    app: BaseApp<SK, PSK, M, H, G, AI, QReq, QRes>,
    listen_addr: SocketAddr,
    router: Router<RestState<SK, PSK, M, H, G, AI, QReq, QRes>>,
    tendermint_rpc_address: Url,
) -> anyhow::Result<()> {
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

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    tracing::info!("REST server running at {}", listen_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
