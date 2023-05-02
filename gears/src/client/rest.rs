use rocket::{
    fairing::{Fairing, Kind},
    figment::Figment,
    get,
    http::Header,
    routes, Config, Request, Response,
};
use serde::{Deserialize, Serialize};
use tendermint_informal::node::Info;
use tendermint_rpc::{Client, HttpClient};

use crate::{app::BaseApp, TM_ADDRESS};

const DEFAULT_SOCKET: u16 = 1317;

#[derive(Serialize, Deserialize)]
struct NodeInfoResponse {
    #[serde(rename = "default_node_info")]
    node_info: Info,
    //TODO: application_version
}

#[get("/cosmos/base/tendermint/v1beta1/node_info")]
async fn node_info() -> String {
    let client = HttpClient::new(TM_ADDRESS)
        .expect("tendermint should be running and accepting connections");

    get_node_status(client).await
}

pub async fn get_node_status(client: HttpClient) -> String {
    let res = client
        .status()
        .await
        .expect("tendermint should respond with no error");

    let node_info = NodeInfoResponse {
        node_info: res.node_info,
    };
    serde_json::to_string_pretty(&node_info).expect("tendermint response should be valid")
}

fn rocket_launch(app: BaseApp) {
    // Disable rocket catching signals to prevent it interfering with the rest
    // of the app. e.g. If this isn't done then rocket shuts down when there's
    // an interrupt but the rest of the app keeps running.
    let figment = Figment::from(Config {
        shutdown: rocket::config::Shutdown {
            ctrlc: false,
            #[cfg(unix)]
            signals: { std::collections::HashSet::new() },
            ..Default::default()
        },
        ..Config::default()
    })
    .merge(("port", DEFAULT_SOCKET));

    let rocket = rocket::custom(figment)
        .manage(app)
        .mount("/", routes![node_info])
        .attach(CORS)
        .launch();

    rocket::execute(async move { rocket.await })
        .expect("the server will only stop when the application is terminated");
}

pub fn run_rest_server(app: BaseApp) {
    std::thread::spawn(move || rocket_launch(app));
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
