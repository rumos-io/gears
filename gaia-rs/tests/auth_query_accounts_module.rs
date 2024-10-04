#![cfg(feature = "it")]

use auth::cli::query::{AccountsCommand, AuthCommands, AuthQueryCli, AuthQueryResponse};
use gaia_rs::{client::GaiaQueryCommands, query::GaiaQueryResponse};
use gears::{extensions::testing::UnwrapTesting, types::account::Account};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

#[test]
fn module_accounts_query() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    let cmd = GaiaQueryCommands::Auth(AuthQueryCli {
        command: AuthCommands::Accounts(AccountsCommand { pagination: None }),
    });

    let result = gaia.query(cmd)?;

    println!("{}", serde_json::to_string_pretty(&result).unwrap_test());

    let expected = r#"
[
    {
    "@type": "/cosmos.auth.v1beta1.ModuleAccount",
    "base_account": {
        "address": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh",
        "pub_key": null,
        "account_number": "0",
        "sequence": "0"
    },
    "name": "bonded_tokens_pool",
    "permissions": [
        "burner",
        "staking"
    ]
    },
    {
    "@type": "/cosmos.auth.v1beta1.ModuleAccount",
    "base_account": {
        "address": "cosmos1tygms3xhhs3yv487phx3dw4a95jn7t7lpm470r",
        "pub_key": null,
        "account_number": "1",
        "sequence": "0"
    },
    "name": "not_bonded_tokens_pool",
    "permissions": [
        "burner",
        "staking"
    ]
    },
    {
    "@type": "/cosmos.auth.v1beta1.ModuleAccount",
    "base_account": {
        "address": "cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta",
        "pub_key": null,
        "account_number": "13",
        "sequence": "0"
    },
    "name": "fee_collector",
    "permissions": []
    }
]"#;

    let expected: Vec<Account> = serde_json::from_str(expected).unwrap_test();

    let result = match result {
        GaiaQueryResponse::Auth(auth_query_response) => match auth_query_response {
            AuthQueryResponse::Accounts(query_accounts_response) => query_accounts_response
                .accounts
                .into_iter()
                .filter(|this| match this {
                    Account::Base(_) => false,
                    Account::Module(_) => true,
                })
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    assert_eq!(result, expected);

    Ok(())
}
