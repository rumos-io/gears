use std::str::FromStr;

use cosmwasm_std::Uint256;
use ibc_proto::{cosmos::base::v1beta1::Coin as RawCoin, protobuf::Protobuf};
use proto_types::Denom;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Coin defines a token with a denomination and an amount.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Coin {
    pub denom: proto_types::Denom,
    pub amount: ::cosmwasm_std::Uint256,
}

impl TryFrom<RawCoin> for Coin {
    type Error = Error;

    fn try_from(value: RawCoin) -> Result<Self, Self::Error> {
        let denom = value
            .denom
            .try_into()
            .map_err(|_| Error::Coin(String::from("coin error")))?;
        let amount = Uint256::from_str(&value.amount)
            .map_err(|_| Error::Coin(String::from("coin error")))?;

        Ok(Coin { denom, amount })
    }
}

impl From<Coin> for RawCoin {
    fn from(value: Coin) -> RawCoin {
        RawCoin {
            denom: value.denom.to_string(),
            amount: value.amount.to_string(),
        }
    }
}

impl Protobuf<RawCoin> for Coin {}

impl FromStr for Coin {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // get the index at which amount ends and denom starts
        let i = input
            .find(|c: char| !c.is_numeric())
            .unwrap_or_else(|| input.len());

        let amount = input[..i]
            .parse::<Uint256>()
            .map_err(|e| Error::Coin(String::from(format!("coin error: {}", e))))?;

        let denom = input[i..]
            .parse::<Denom>()
            .map_err(|e| Error::Coin(String::from(format!("coin error: {}", e))))?;

        Ok(Coin { denom, amount })
    }
}

// Represents a list of coins with the following properties:
// - Contains at least one coin
// - All coin amounts are positive
// - No duplicate denominations
// - Sorted lexicographically
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SendCoins(Vec<Coin>);

impl SendCoins {
    pub fn new(coins: Vec<Coin>) -> Result<SendCoins, Error> {
        Self::validate_coins(&coins)?;

        Ok(SendCoins(coins))
    }

    // Checks that the SendCoins are sorted, have positive amount, with a valid and unique
    // denomination (i.e no duplicates). Otherwise, it returns an error.
    // A valid list of coins satisfies:
    // - Contains at least one coin
    // - All amounts are positive
    // - No duplicate denominations
    // - Sorted lexicographically
    // TODO: implement ordering on coins or denominations so that conversion to string can be avoided
    fn validate_coins(coins: &Vec<Coin>) -> Result<(), Error> {
        if coins.is_empty() {
            return Err(Error::Coins(String::from("list of coins is empty")));
        }

        if coins[0].amount.is_zero() {
            return Err(Error::Coins(String::from("coin amount must be positive")));
        };

        let mut previous_denom = coins[0].denom.to_string();

        for coin in &coins[1..] {
            if coin.amount.is_zero() {
                return Err(Error::Coins(String::from("coin amount must be positive")));
            };

            // Less than to ensure lexicographical ordering
            // Equality to ensure that there are no duplications
            if coin.denom.to_string() <= previous_denom {
                return Err(Error::Coins(String::from(
                    "coins are not sorted and/or contain duplicates",
                )));
            }

            previous_denom = coin.denom.to_string();
        }

        return Ok(());
    }
}

impl From<SendCoins> for Vec<Coin> {
    fn from(coins: SendCoins) -> Vec<Coin> {
        coins.0
    }
}

impl IntoIterator for SendCoins {
    type Item = Coin;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for SendCoins {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let coin_strings = input.split(",");
        let mut coins = vec![];

        for coin in coin_strings {
            let coin = Coin::from_str(coin)?;
            coins.push(coin);
        }

        Self::new(coins)
    }
}

#[cfg(test)]
mod tests {

    use cosmwasm_std::Uint256;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn coin_from_string_successes() {
        let raw = "32454uatom";
        let coin = raw.parse::<Coin>().unwrap();
        assert_eq!(
            Coin {
                denom: String::from("uatom").try_into().unwrap(),
                amount: "32454".try_into().unwrap()
            },
            coin
        );

        let raw = "0uatom";
        let coin = raw.parse::<Coin>().unwrap();
        assert_eq!(
            Coin {
                denom: String::from("uatom").try_into().unwrap(),
                amount: "0".try_into().unwrap()
            },
            coin
        );

        let raw = "0001uatom";
        let coin = raw.parse::<Coin>().unwrap();
        assert_eq!(
            Coin {
                denom: String::from("uatom").try_into().unwrap(),
                amount: "1".try_into().unwrap()
            },
            coin
        );

        let raw = "12uatom56";
        let coin = raw.parse::<Coin>().unwrap();
        assert_eq!(
            Coin {
                denom: String::from("uatom56").try_into().unwrap(),
                amount: "12".try_into().unwrap()
            },
            coin
        );
    }

    #[test]
    fn coin_from_string_failures() {
        let raw = "32454-uatom";
        raw.parse::<Coin>().unwrap_err();

        let raw = "-32454uatom";
        raw.parse::<Coin>().unwrap_err();

        let raw = " 54uatom";
        raw.parse::<Coin>().unwrap_err();

        let raw = "54 uatom";
        raw.parse::<Coin>().unwrap_err();

        let raw = "54uatom ";
        raw.parse::<Coin>().unwrap_err();
    }

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
        SendCoins::new(coins).unwrap();

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
        SendCoins::new(coins).unwrap();

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
        SendCoins::new(coins).unwrap();
    }

    #[test]
    fn validate_coins_fail() {
        // empty
        let coins = vec![];
        let err = SendCoins::new(coins).unwrap_err();
        assert_eq!(err.to_string(), String::from("list of coins is empty"));

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
        let err = SendCoins::new(coins).unwrap_err();
        assert_eq!(
            err.to_string(),
            String::from("coins are not sorted and/or contain duplicates")
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
        let err = SendCoins::new(coins).unwrap_err();
        assert_eq!(
            err.to_string(),
            String::from("coins are not sorted and/or contain duplicates")
        );

        // not positive
        let coins = vec![Coin {
            denom: String::from("truer").try_into().unwrap(),
            amount: Uint256::zero(),
        }];
        let err = SendCoins::new(coins).unwrap_err();
        assert_eq!(
            err.to_string(),
            String::from("coin amount must be positive")
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
        let err = SendCoins::new(coins).unwrap_err();
        assert_eq!(
            err.to_string(),
            String::from("coin amount must be positive")
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
        let err = SendCoins::new(coins).unwrap_err();
        assert_eq!(
            err.to_string(),
            String::from("coins are not sorted and/or contain duplicates")
        );
    }

    #[test]
    fn coins_from_string_successes() {
        let raw_coins = "100atom,30uatom";
        SendCoins::from_str(raw_coins).unwrap();
    }

    #[test]
    fn coins_from_string_failure() {
        let raw_coins = "100atom,30uatom,";
        SendCoins::from_str(raw_coins).unwrap_err();

        // no space at beginning
        let raw_coins = " 100atom,30uatom";
        SendCoins::from_str(raw_coins).unwrap_err();

        // no space at separator
        let raw_coins = "100atom, 30uatom";
        SendCoins::from_str(raw_coins).unwrap_err();

        // no space at end
        let raw_coins = "100atom,30uatom ";
        SendCoins::from_str(raw_coins).unwrap_err();
    }
}
