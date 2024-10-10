#![cfg(feature = "it")]

use auth::cli::query::{AccountsCommand, AuthCommands, AuthQueryCli, AuthQueryResponse};
use gaia_rs::{client::GaiaQueryCommands, query::GaiaQueryResponse};
use gears::{extensions::testing::UnwrapTesting, types::account::Account};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

#[test]
fn base_accounts_query() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    let cmd = GaiaQueryCommands::Auth(AuthQueryCli {
        command: AuthCommands::Accounts(AccountsCommand { pagination: None }),
    });

    let result = gaia.query(cmd)?;

    println!("{}", serde_json::to_string_pretty(&result).unwrap_test());

    let expected = r#"
    [
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos19k5n7f35e4tskjcm2peujta0e3rvszmmzlhjej",
          "pub_key": null,
          "account_number": "13",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos18mjeafdpsgdgu0tmt39zggx20h7994hpd6wkpx",
          "pub_key": null,
          "account_number": "4",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1f78mzqldvqmpxxf5xf0354hu3cx7n3xrptyql9",
          "pub_key": null,
          "account_number": "11",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
          "pub_key": {
            "@type": "/cosmos.crypto.secp256k1.PubKey",
            "key": "AvUEsFHbsr40nTSmWh7CWYRZHGwf4cpRLtJlaRO4VAoq"
          },
          "account_number": "3",
          "sequence": "1"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1nxyyhwcfzzptad0trg6suh6j650npjlxxc73ay",
          "pub_key": null,
          "account_number": "5",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos15rteevxs5vsxfaj5qukh0hhuecgkel8s45g4dt",
          "pub_key": null,
          "account_number": "9",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1ks9zdcycywxfcj880m3rlyctsfhfsh87fc7cm3",
          "pub_key": null,
          "account_number": "6",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1kn7ccmk9k2q024l66efw39mat4ys2cvxc2kh67",
          "pub_key": null,
          "account_number": "12",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1mf4pu0zmzlng5rn8m0nmju9auxujns6jswcxwr",
          "pub_key": null,
          "account_number": "8",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1ly29f22fpeqsafa03tpnf7huyewv9vl2dvamxp",
          "pub_key": null,
          "account_number": "10",
          "sequence": "0"
        },
        {
          "@type": "/cosmos.auth.v1beta1.BaseAccount",
          "address": "cosmos1luuusx6cura35dl4ztezd9x2pjea3js0xvvmc8",
          "pub_key": null,
          "account_number": "7",
          "sequence": "0"
        }
      ]
    "#;

    let expected: Vec<Account> = serde_json::from_str(expected).unwrap_test();

    let result = match result {
        GaiaQueryResponse::Auth(auth_query_response) => match auth_query_response {
            AuthQueryResponse::Accounts(query_accounts_response) => query_accounts_response
                .accounts
                .into_iter()
                .filter(|this| match this {
                    Account::Base(_) => true,
                    Account::Module(_) => false,
                })
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    assert_eq!(result, expected);

    Ok(())
}
