pub mod app;
pub mod baseapp;
pub mod client;
pub mod crypto;
pub mod error;
pub mod store;
pub mod types;
pub mod utils;
pub mod x;

const TM_ADDRESS: &str = "http://localhost:26657"; // used by rest service when proxying requests to tendermint // TODO: this needs to be configurable
