use rocket::{
    fairing::{AdHoc, Fairing, Kind},
    figment::Figment,
    http::{Accept, Header},
    routes, Config, Request, Response as RocketResponse,
};

use super::handlers::{
    get_balances, get_balances_by_denom, node_info, staking_params, supply, txs,
};
use crate::app::BaseApp;

const DEFAULT_SOCKET: u16 = 1317;

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
        .mount(
            "/",
            routes![
                node_info,
                txs,
                staking_params,
                supply,
                get_balances,
                get_balances_by_denom
            ],
        )
        .attach(CORS)
        // Replace "accept" header to force rocket to return json errors rather than the default HTML.
        // Doesn't work if request is malformed (HTML is returned) see https://github.com/SergioBenitez/Rocket/issues/2129
        .attach(AdHoc::on_request("Accept Rewriter", |req, _| {
            Box::pin(async move {
                req.replace_header(Accept::JSON);
            })
        }))
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

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut RocketResponse<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
