use std::collections::{HashMap, HashSet};

use cosmwasm_std::Uint256;
use ibc_proto::cosmos::tx::v1beta1::{AuthInfo, TxBody};
use serde::de::IntoDeserializer;
use structopt::clap::App;

use crate::error::AppError;

// /// MsgSend represents a message to send coins from one account to another.
// #[derive(Clone, PartialEq, ::prost::Message)]
// pub struct MsgSend {
//     #[prost(address, tag = "1")]
//     pub from_address: proto_types::AccAddress,
//     #[prost(address, tag = "2")]
//     pub to_address: proto_types::AccAddress,
//     #[prost(message, repeated, tag = "3")]
//     pub amount: Vec<Coin>,
// }

/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBalanceRequest {
    /// address is the address to query balances for.
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
    /// denom is the coin denom to query balances for.
    #[prost(denom, tag = "2")]
    pub denom: proto_types::Denom,
}

/// QueryBalanceRequest is the request type for the Query/AllBalances RPC method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllBalancesRequest {
    /// address is the address to query balances for.
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: Option<ibc_proto::cosmos::base::query::v1beta1::PageRequest>,
}

/// BaseAccount defines a base account type. It contains all the necessary fields
/// for basic account functionality. Any custom account type should extend this
/// type for additional functionality (e.g. vesting).
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseAccount {
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
    #[prost(message, optional, tag = "2")]
    pub pub_key: Option<ibc_proto::google::protobuf::Any>,
    #[prost(uint64, tag = "3")]
    pub account_number: u64,
    #[prost(uint64, tag = "4")]
    pub sequence: u64,
}

/// ModuleAccount defines an account for modules that holds coins on a pool.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleAccount {
    #[prost(message, required, tag = "1")]
    pub base_account: BaseAccount,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub permissions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    #[prost(address, tag = "1")]
    pub address: proto_types::AccAddress,
}

/// Tx is the standard type used for broadcasting transactions.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tx {
    /// body is the processable content of the transaction
    #[prost(message, required, tag = "1")]
    pub body: TxBody,
    /// auth_info is the authorization related content of the transaction,
    /// specifically signers, signer modes and fee
    #[prost(message, required, tag = "2")]
    pub auth_info: AuthInfo,
    /// signatures is a list of signatures that matches the length and order of
    /// AuthInfo's signer_infos to allow connecting signature meta information like
    /// public key and signing mode by position.
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub signatures: Vec<Vec<u8>>,
}

/// Coin defines a token with a denomination and an amount.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(denom, tag = "1")]
    pub denom: proto_types::Denom,
    #[prost(uint256, tag = "2")]
    pub amount: ::cosmwasm_std::Uint256,
}

// Checks that the Coins are sorted, have positive amount, with a valid and unique
// denomination (i.e no duplicates). Otherwise, it returns an error.
// A valid list of coins satisfies:
// Contains at least one coin
// All amounts are positive
// No duplicate denominations
// Sorted lexicographically
// TODO: implement ordering on coins or denominations
pub fn validate_coins(coins: &Vec<Coin>) -> Result<(), AppError> {
    if coins.is_empty() {
        return Err(AppError::Coins(String::from("list of coins is empty")));
    }

    if coins[0].amount.is_zero() {
        return Err(AppError::Coins(String::from(
            "coin amount must be positive",
        )));
    };

    let mut previous_denom = coins[0].denom.to_string();

    for coin in &coins[1..] {
        if coin.amount.is_zero() {
            return Err(AppError::Coins(String::from(
                "coin amount must be positive",
            )));
        };

        // Less than to ensure lexicographical ordering
        // Equality to ensure that there are no duplications
        if coin.denom.to_string() <= previous_denom {
            return Err(AppError::Coins(String::from(
                "coins are not sorted and/or contain duplicates",
            )));
        }

        previous_denom = coin.denom.to_string();
    }

    return Ok(());
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;

    #[test]
    fn validate_coins_success() {
        let coins = vec![
            Coin {
                denom: String::from("atom").try_into().unwrap(),
                amount: Uint256::one(),
            },
            Coin {
                denom: String::from("atom1").try_into().unwrap(),
                amount: Uint256::one(),
            },
        ];
        validate_coins(&coins).unwrap();

        // ibc denoms
        let coins = vec![
            Coin {
                denom: String::from(
                    "ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2",
                )
                .try_into()
                .unwrap(),
                amount: Uint256::one(),
            },
            Coin {
                denom: String::from(
                    "ibc/876563AAAACF739EB061C67CDB5EDF2B7C9FD4AA9D876450CC21210807C2820A",
                )
                .try_into()
                .unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        validate_coins(&coins).unwrap();

        // prefix lexicographical ordering
        let coins = vec![
            Coin {
                denom: String::from("big").try_into().unwrap(),
                amount: Uint256::one(),
            },
            Coin {
                denom: String::from("bigger").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        validate_coins(&coins).unwrap();
    }

    #[test]
    fn validate_coins_fail() {
        // empty
        let coins = vec![];
        let err = validate_coins(&coins).unwrap_err();
        assert_eq!(err, AppError::Coins(String::from("list of coins is empty")));

        // not sorted
        let coins = vec![
            Coin {
                denom: String::from("tree").try_into().unwrap(),
                amount: Uint256::one(),
            },
            Coin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            Coin {
                denom: String::from("mineral").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        let err = validate_coins(&coins).unwrap_err();
        assert_eq!(
            err,
            AppError::Coins(String::from(
                "coins are not sorted and/or contain duplicates"
            ))
        );

        // not sorted 2
        let coins = vec![
            Coin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::one(),
            },
            Coin {
                denom: String::from("true").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            Coin {
                denom: String::from("mineral").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        let err = validate_coins(&coins).unwrap_err();
        assert_eq!(
            err,
            AppError::Coins(String::from(
                "coins are not sorted and/or contain duplicates"
            ))
        );

        // not positive
        let coins = vec![Coin {
            denom: String::from("truer").try_into().unwrap(),
            amount: Uint256::zero(),
        }];
        let err = validate_coins(&coins).unwrap_err();
        assert_eq!(
            err,
            AppError::Coins(String::from("coin amount must be positive"))
        );

        // not all positive
        let coins = vec![
            Coin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::one(),
            },
            Coin {
                denom: String::from("true").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            Coin {
                denom: String::from("truer").try_into().unwrap(),
                amount: Uint256::zero(),
            },
        ];
        let err = validate_coins(&coins).unwrap_err();
        assert_eq!(
            err,
            AppError::Coins(String::from("coin amount must be positive"))
        );

        // duplicate denomination
        let coins = vec![
            Coin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::one(),
            },
            Coin {
                denom: String::from("truer").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            Coin {
                denom: String::from("truer").try_into().unwrap(),
                amount: Uint256::one(),
            },
        ];
        let err = validate_coins(&coins).unwrap_err();
        assert_eq!(
            err,
            AppError::Coins(String::from(
                "coins are not sorted and/or contain duplicates"
            ))
        );
    }
}
