use auth::cli::query::{AccountCommand, AuthCommands, AuthQueryCli, AuthQueryResponse};
use gaia_rs::{
    client::{GaiaQueryCommands, WrappedGaiaQueryCommands},
    query::GaiaQueryResponse,
    GaiaCoreClient,
};
use gears::{
    commands::client::query::{run_query, QueryCommand},
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
    types::address::AccAddress,
    types::{
        account::{Account, BaseAccount},
        query::account::QueryAccountResponse,
    },
};

use utilities::run_gaia_and_tendermint;

#[path = "./utilities.rs"]
mod utilities;

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn account_query() -> anyhow::Result<()> {
    let (_tendermint, _server_thread) = run_gaia_and_tendermint()?;

    let acc_adress = AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
        .expect("Valid value");

    let query = AccountCommand {
        address: acc_adress.clone(),
    };

    let cmd = QueryCommand {
        node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
        height: None,
        inner: WrappedGaiaQueryCommands(GaiaQueryCommands::Auth(AuthQueryCli {
            command: AuthCommands::Account(query),
        })),
    };

    let result = run_query(cmd, &GaiaCoreClient)?;

    let expected = GaiaQueryResponse::Auth(AuthQueryResponse::Account(QueryAccountResponse {
        account: Account::Base(BaseAccount {
            address: acc_adress,
            pub_key: None,
            account_number: 0,
            sequence: 0,
        }),
    }));

    assert_eq!(result, expected);

    Ok(())
}
