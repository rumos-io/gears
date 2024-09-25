use std::net::SocketAddr;

use tonic::transport::server::Router;
use tower_layer::Identity;

use crate::runtime::runtime;

mod error;
pub mod health;
pub mod tx;

pub fn run_grpc_server(router: Router<Identity>, listen_addr: SocketAddr) {
    std::thread::spawn(move || {
        let result = runtime().block_on(launch(router, listen_addr));
        if let Err(err) = result {
            panic!("Failed to run gRPC server with err: {}", err)
        }
    });
}

// #[allow(dead_code)]
// trait GService:
//     Service<
//         http::Request<Body>,
//         Response = http::Response<BoxBody>,
//         Error = Infallible,
//         Future = dyn Send + 'static,
//     > + NamedService
//     + Clone
//     + Send
//     + 'static
// {
// }

async fn launch(
    router: Router<Identity>,
    listen_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("gRPC server running at {}", listen_addr);
    router.serve(listen_addr).await?;
    Ok(())
}
