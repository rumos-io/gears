use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use proto_messages::cosmos::tx::v1beta1::Message;
// use rocket::{
//     fairing::{AdHoc, Fairing, Kind},
//     figment::Figment,
//     http::{Accept, Header},
//     routes, Config, Request, Response as RocketResponse,
// };
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{hash::Hash, net::SocketAddr};
use store_crate::StoreKey;
use strum::IntoEnumIterator;
use tokio::runtime::Runtime;

use crate::{
    baseapp::{
        ante::{AuthKeeper, BankKeeper},
        BaseApp, Handler,
    },
    client::rest::handlers::node_info,
    x::params::ParamsSubspaceKey,
};

// use super::{
//     handlers::{node_info, staking_params, txs},
//     pagination::Pagination,
// };

// fn rocket_launch<
//     SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
//     PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
//     M: Message,
//     BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
//     AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
//     H: Handler<M, SK, G> + 'static,
//     G: DeserializeOwned + Clone + Send + Sync + 'static,
// >(
//     app: BaseApp<SK, PSK, M, BK, AK, H, G>,
//     port: u16,
// ) {
//     // Disable rocket catching signals to prevent it interfering with the rest
//     // of the app. e.g. If this isn't done then rocket shuts down when there's
//     // an interrupt but the rest of the app keeps running.
//     let figment = Figment::from(Config {
//         shutdown: rocket::config::Shutdown {
//             ctrlc: false,
//             #[cfg(unix)]
//             signals: { std::collections::HashSet::new() },
//             ..Default::default()
//         },
//         ..Config::default()
//     })
//     .merge(("port", port));

//     let a = txs::<M>(
//         "dsdsd",
//         Pagination {
//             offset: todo!(),
//             limit: todo!(),
//         },
//     );

//     let rocket = rocket::custom(figment)
//         .manage(app)
//         .mount(
//             "/",
//             routes![
//                 node_info,
//                 //txs,
//                 staking_params,
//                 // supply,
//                 // get_balances,
//                 // get_balances_by_denom
//             ],
//         )
//         .attach(CORS)
//         // Replace "accept" header to force rocket to return json errors rather than the default HTML.
//         // Doesn't work if request is malformed (HTML is returned) see https://github.com/SergioBenitez/Rocket/issues/2129
//         .attach(AdHoc::on_request("Accept Rewriter", |req, _| {
//             Box::pin(async move {
//                 req.replace_header(Accept::JSON);
//             })
//         }))
//         .launch();

//     rocket::execute(async move { rocket.await })
//         .expect("the server will only stop when the application is terminated");
// }

// pub fn run_rest_server<
//     SK: Hash + Eq + IntoEnumIterator + StoreKey + Clone + Send + Sync + 'static,
//     PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
//     M: Message,
//     BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
//     AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
//     H: Handler<M, SK, G> + 'static,
//     G: DeserializeOwned + Clone + Send + Sync + 'static,
// >(
//     app: BaseApp<SK, PSK, M, BK, AK, H, G>,
//     port: u16,
// ) {
//     std::thread::spawn(move || rocket_launch(app, port));
// }

// pub struct CORS;

// #[rocket::async_trait]
// impl Fairing for CORS {
//     fn info(&self) -> rocket::fairing::Info {
//         rocket::fairing::Info {
//             name: "Add CORS headers to responses",
//             kind: Kind::Response,
//         }
//     }

//     async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut RocketResponse<'r>) {
//         response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
//         response.set_header(Header::new(
//             "Access-Control-Allow-Methods",
//             "POST, GET, PATCH, OPTIONS",
//         ));
//         response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
//         response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
//     }
// }

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
) {
    std::thread::spawn(move || {
        Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(launch(port));
    });
}

// TODO:
// 1. CORS
// 2. Replace "accept" header to force rocket to return json errors rather than the default HTML.
// 3. what happens if a route panics?
async fn launch(port: u16) {
    let app = Router::new().route("/cosmos/base/tendermint/v1beta1/node_info", get(node_info));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
