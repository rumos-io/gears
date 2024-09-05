use cosmwasm_std::Uint256;

use crate::types::base::{coin::UnsignedCoin, errors::CoinsError};

use super::Coins;

pub type UnsignedCoins = Coins<Uint256, UnsignedCoin>;

impl UnsignedCoins {
    pub fn checked_add(&self, other: &Self) -> Result<Self, CoinsError> {
        Self::new(self.storage.iter().chain(other.storage.iter()).cloned())
    }

    pub fn is_all_gte<'a>(&self, other: impl IntoIterator<Item = &'a UnsignedCoin>) -> bool {
        let other = other.into_iter().collect::<Vec<_>>();

        if other.is_empty() {
            return true;
        }

        for coin in other {
            if coin.amount >= self.amount_of(&coin.denom) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {

    use cosmwasm_std::Uint256;
    use extensions::testing::UnwrapTesting;
    use std::str::FromStr;

    use crate::types::base::errors::CoinsError;

    use super::*;

    #[test]
    fn coin_from_string_successes() {
        let raw = "32454uatom";
        let coin = raw.parse::<UnsignedCoin>().unwrap_test();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom").try_into().unwrap_test(),
                amount: "32454".try_into().unwrap_test()
            },
            coin
        );

        let raw = "0uatom";
        let coin = raw.parse::<UnsignedCoin>().unwrap_test();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom").try_into().unwrap_test(),
                amount: "0".try_into().unwrap_test()
            },
            coin
        );

        let raw = "0001uatom";
        let coin = raw.parse::<UnsignedCoin>().unwrap_test();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom").try_into().unwrap_test(),
                amount: "1".try_into().unwrap_test()
            },
            coin
        );

        let raw = "12uatom56";
        let coin = raw.parse::<UnsignedCoin>().unwrap_test();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom56").try_into().unwrap_test(),
                amount: "12".try_into().unwrap_test()
            },
            coin
        );
    }

    #[test]
    fn coin_from_string_failures() {
        let raw = "32454-uatom";
        raw.parse::<UnsignedCoin>().unwrap_err();

        let raw = "-32454uatom";
        raw.parse::<UnsignedCoin>().unwrap_err();

        let raw = " 54uatom";
        raw.parse::<UnsignedCoin>().unwrap_err();

        let raw = "54 uatom";
        raw.parse::<UnsignedCoin>().unwrap_err();

        let raw = "54uatom ";
        raw.parse::<UnsignedCoin>().unwrap_err();
    }

    #[test]
    fn validate_coins_success() {
        let coins = vec![
            UnsignedCoin {
                denom: String::from("atom").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("atom1").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
        ];
        UnsignedCoins::new(coins).unwrap_test();

        // ibc denoms
        let coins = vec![
            UnsignedCoin {
                denom: String::from(
                    "ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2",
                )
                .try_into()
                .unwrap_test(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from(
                    "ibc/876563AAAACF739EB061C67CDB5EDF2B7C9FD4AA9D876450CC21210807C2820A",
                )
                .try_into()
                .unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
        ];
        UnsignedCoins::new(coins).unwrap_test();

        // prefix lexicographical ordering
        let coins = vec![
            UnsignedCoin {
                denom: String::from("big").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("bigger").try_into().unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
        ];
        UnsignedCoins::new(coins).unwrap_test();
    }

    #[test]
    fn validate_coins_fail() {
        // empty
        let coins = vec![];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::EmptyList));

        // not positive
        let coins = vec![UnsignedCoin {
            denom: String::from("truer").try_into().unwrap_test(),
            amount: Uint256::zero(),
        }];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::InvalidAmount));

        // not all positive
        let coins = vec![
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("true").try_into().unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
            UnsignedCoin {
                denom: String::from("truer").try_into().unwrap_test(),
                amount: Uint256::zero(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::InvalidAmount));

        // duplicate denomination
        let coins = vec![
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("truer").try_into().unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
            UnsignedCoin {
                denom: String::from("truer").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::Duplicates));

        // not sorted
        let coins = vec![
            UnsignedCoin {
                denom: String::from("tree").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
            UnsignedCoin {
                denom: String::from("mineral").try_into().unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::Unsorted));

        // not sorted 2
        let coins = vec![
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap_test(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("true").try_into().unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
            UnsignedCoin {
                denom: String::from("mineral").try_into().unwrap_test(),
                amount: Uint256::from_str("3").unwrap_test(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::Unsorted));
    }

    #[test]
    fn coins_from_string_successes() {
        let raw_coins = "100atom,30uatom";
        UnsignedCoins::from_str(raw_coins).unwrap_test();
    }

    #[test]
    fn coins_from_string_failure() {
        let raw_coins = "100atom,30uatom,";
        UnsignedCoins::from_str(raw_coins).unwrap_err();

        // no space at beginning
        let raw_coins = " 100atom,30uatom";
        UnsignedCoins::from_str(raw_coins).unwrap_err();

        // no space at separator
        let raw_coins = "100atom, 30uatom";
        UnsignedCoins::from_str(raw_coins).unwrap_err();

        // no space at end
        let raw_coins = "100atom,30uatom ";
        UnsignedCoins::from_str(raw_coins).unwrap_err();
    }
}
