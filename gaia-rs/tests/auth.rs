#![cfg(feature = "it")]

use auth::{
    cli::query::{AccountCommand, AuthCommands, AuthQueryCli, AuthQueryResponse},
    query::QueryAccountResponse,
};
use gaia_rs::{
    client::{GaiaQueryCommands, WrappedGaiaQueryCommands},
    query::GaiaQueryResponse,
    GaiaCoreClient,
};
use gears::{
    commands::client::query::{run_query, QueryCommand},
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
    types::account::{Account, BaseAccount},
};

use utilities::{acc_address, tendermint};

#[path = "./utilities.rs"]
mod utilities;

#[test]
fn account_query() -> anyhow::Result<()> {
    let _tendermint = tendermint();

    let query = AccountCommand {
        address: acc_address(),
    };

    let cmd = QueryCommand {
        node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
        height: None,
        inner: WrappedGaiaQueryCommands(GaiaQueryCommands::Auth(AuthQueryCli {
            command: AuthCommands::Account(query),
        })),
    };

    let result = run_query(cmd, &GaiaCoreClient)?;

    // Note that this may be issue 'cause of sequence
    let expected = GaiaQueryResponse::Auth(AuthQueryResponse::Account(QueryAccountResponse {
        account: Some(Account::Base(BaseAccount {
            address: acc_address(),
            pub_key: None,
            account_number: 2,
            sequence: 0,
        })),
    }));

    assert_eq!(result, expected);

    Ok(())
}
