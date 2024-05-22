use std::convert::Infallible;

use tonic::{
    body::BoxBody,
    server::NamedService,
    transport::{server::Router, Body},
};
use tower_layer::Identity;
use tower_service::Service;

use crate::runtime::runtime;

mod error;

pub fn run_grpc_server(router: Router<Identity>) {
    std::thread::spawn(move || {
        let result = runtime().block_on(launch(router));
        if let Err(err) = result {
            panic!("Failed to run gRPC server with err: {}", err)
        }
    });
}

trait GService:
    Service<
        http::Request<Body>,
        Response = http::Response<BoxBody>,
        Error = Infallible,
        Future = dyn Send + 'static,
    > + NamedService
    + Clone
    + Send
    + 'static
{
}

async fn launch(router: Router<Identity>) -> Result<(), Box<dyn std::error::Error>> {
    let address = "127.0.0.1:8080"
        .parse()
        .expect("hard coded address is valid");

    tracing::info!("gRPC server running at {}", address);
    router.serve(address).await?;
    Ok(())
}
