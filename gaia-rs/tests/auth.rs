// #![cfg(feature = "it")]

use auth::{
    cli::query::{AccountCommand, AuthCommands, AuthQueryCli, AuthQueryResponse},
    query::QueryAccountResponse,
};
use gaia_rs::{client::GaiaQueryCommands, query::GaiaQueryResponse};
use gears::types::account::{Account, BaseAccount};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

#[test]
fn account_query() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    let addr = GaiaNode::validator_account();

    let cmd = GaiaQueryCommands::Auth(AuthQueryCli {
        command: AuthCommands::Account(AccountCommand {
            address: addr.clone(),
        }),
    });

    let result = gaia.query(cmd)?;

    // Note that this may be issue 'cause of sequence
    let expected = GaiaQueryResponse::Auth(AuthQueryResponse::Account(QueryAccountResponse {
        account: Some(Account::Base(BaseAccount {
            address: addr,
            pub_key: None,
            account_number: 2,
            sequence: 0,
        })),
    }));

    assert_eq!(result, expected);

    Ok(())
}
