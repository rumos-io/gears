#![cfg(feature = "it")]

use bank::{
    cli::query::{BankCommands as BankQueryCommands, BankQueryCli, BankQueryResponse},
    types::query::QueryDenomsMetadataResponse,
};
use gaia_rs::{client::GaiaQueryCommands, query::GaiaQueryResponse};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

#[test]
fn denom_query() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    let cmd = GaiaQueryCommands::Bank(BankQueryCli {
        command: BankQueryCommands::DenomMetadata { pagination: None },
    });

    let result = gaia.query(cmd)?;

    let expected = GaiaQueryResponse::Bank(BankQueryResponse::DenomMetadata(
        QueryDenomsMetadataResponse {
            metadatas: Vec::new(),
            pagination: None,
        },
    ));

    assert_eq!(result, expected);

    Ok(())
}
