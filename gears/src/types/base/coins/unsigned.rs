use std::cmp::Ordering;

use cosmwasm_std::{OverflowError, Uint256};

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

    // TODO: Move this to generic declaration
    /// Substracts matching coins. If the other coins have bigger values or the coins that don't
    /// exists in original set, method returns error. If all coins are identical method returns
    /// error.
    pub fn checked_sub(&self, other: &UnsignedCoins) -> Result<Self, CoinsError> {
        if self.is_all_gte(other.inner()) {
            let coins: Vec<UnsignedCoin> = self
                .checked_calculate_iterate(other.inner(), Uint256::checked_sub)?
                .into_iter()
                // filter zeros after sub
                .filter(|c| !c.amount.is_zero())
                .collect();
            Self::new(coins)
        } else {
            Err(CoinsError::InvalidAmount)
        }
    }

    fn checked_calculate_iterate(
        &self,
        other_coins: &[UnsignedCoin],
        operation: impl Fn(Uint256, Uint256) -> Result<Uint256, OverflowError>,
    ) -> Result<Vec<UnsignedCoin>, CoinsError> {
        let mut i = 0;
        let mut j = 0;
        let self_coins = self.inner();

        let mut result = vec![];
        let self_coins_len = self_coins.len();
        let other_coins_len = other_coins.len();
        while i < self_coins_len || j < other_coins_len {
            if i == self_coins_len {
                result.extend_from_slice(&other_coins[j..]);
                return Ok(result);
            } else if j == other_coins_len {
                result.extend_from_slice(&self_coins[i..]);
                return Ok(result);
            }
            match self_coins[i].denom.cmp(&other_coins[j].denom) {
                Ordering::Less => {
                    result.push(self_coins[i].clone());
                    i += 1;
                }
                Ordering::Equal => {
                    result.push(UnsignedCoin {
                        denom: self_coins[i].denom.clone(),
                        amount: operation(self_coins[i].amount, other_coins[j].amount)
                            .map_err(|_| CoinsError::InvalidAmount)?,
                    });
                    i += 1;
                    j += 1;
                }
                Ordering::Greater => {
                    result.push(other_coins[j].clone());
                    j += 1;
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use cosmwasm_std::Uint256;
    use std::str::FromStr;

    use crate::types::base::errors::CoinsError;

    use super::*;

    #[test]
    fn coin_from_string_successes() {
        let raw = "32454uatom";
        let coin = raw.parse::<UnsignedCoin>().unwrap();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom").try_into().unwrap(),
                amount: "32454".try_into().unwrap()
            },
            coin
        );

        let raw = "0uatom";
        let coin = raw.parse::<UnsignedCoin>().unwrap();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom").try_into().unwrap(),
                amount: "0".try_into().unwrap()
            },
            coin
        );

        let raw = "0001uatom";
        let coin = raw.parse::<UnsignedCoin>().unwrap();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom").try_into().unwrap(),
                amount: "1".try_into().unwrap()
            },
            coin
        );

        let raw = "12uatom56";
        let coin = raw.parse::<UnsignedCoin>().unwrap();
        assert_eq!(
            UnsignedCoin {
                denom: String::from("uatom56").try_into().unwrap(),
                amount: "12".try_into().unwrap()
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
                denom: String::from("atom").try_into().unwrap(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("atom1").try_into().unwrap(),
                amount: Uint256::one(),
            },
        ];
        UnsignedCoins::new(coins).unwrap();

        // ibc denoms
        let coins = vec![
            UnsignedCoin {
                denom: String::from(
                    "ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2",
                )
                .try_into()
                .unwrap(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from(
                    "ibc/876563AAAACF739EB061C67CDB5EDF2B7C9FD4AA9D876450CC21210807C2820A",
                )
                .try_into()
                .unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        UnsignedCoins::new(coins).unwrap();

        // prefix lexicographical ordering
        let coins = vec![
            UnsignedCoin {
                denom: String::from("big").try_into().unwrap(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("bigger").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        UnsignedCoins::new(coins).unwrap();
    }

    #[test]
    fn validate_coins_fail() {
        // empty
        let coins = vec![];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::EmptyList));

        // not positive
        let coins = vec![UnsignedCoin {
            denom: String::from("truer").try_into().unwrap(),
            amount: Uint256::zero(),
        }];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::InvalidAmount));

        // not all positive
        let coins = vec![
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("true").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            UnsignedCoin {
                denom: String::from("truer").try_into().unwrap(),
                amount: Uint256::zero(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::InvalidAmount));

        // duplicate denomination
        let coins = vec![
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("truer").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            UnsignedCoin {
                denom: String::from("truer").try_into().unwrap(),
                amount: Uint256::one(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::Duplicates));

        // not sorted
        let coins = vec![
            UnsignedCoin {
                denom: String::from("tree").try_into().unwrap(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            UnsignedCoin {
                denom: String::from("mineral").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::Unsorted));

        // not sorted 2
        let coins = vec![
            UnsignedCoin {
                denom: String::from("gas").try_into().unwrap(),
                amount: Uint256::one(),
            },
            UnsignedCoin {
                denom: String::from("true").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
            UnsignedCoin {
                denom: String::from("mineral").try_into().unwrap(),
                amount: Uint256::from_str("3").unwrap(),
            },
        ];
        let err = UnsignedCoins::new(coins);
        assert_eq!(err, Err(CoinsError::Unsorted));
    }

    #[test]
    fn coins_from_string_successes() {
        let raw_coins = "100atom,30uatom";
        UnsignedCoins::from_str(raw_coins).unwrap();
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
