#![cfg(feature = "it")]

use auth::{
    cli::query::{AccountCommand, AuthCommands, AuthQueryCli, AuthQueryResponse},
    query::QueryAccountResponse,
};
use gaia_rs::{client::GaiaQueryCommands, query::GaiaQueryResponse};
use gears::{
    crypto::public::PublicKey,
    extensions::testing::UnwrapTesting,
    types::account::{Account, BaseAccount},
};
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
            pub_key: Some(PublicKey::Secp256k1(
                serde_json::from_str(
                    r#"{ "key": "AvUEsFHbsr40nTSmWh7CWYRZHGwf4cpRLtJlaRO4VAoq" }"#,
                )
                .unwrap_test(),
            )),
            account_number: 3,
            sequence: 1,
        })),
    }));

    assert_eq!(result, expected);

    Ok(())
}
