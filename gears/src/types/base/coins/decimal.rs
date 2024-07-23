use std::cmp::Ordering;

use cosmwasm_std::{Decimal256, OverflowError};

use crate::types::base::{
    coin::{DecimalCoin, UnsignedCoin},
    errors::CoinsError,
};

use super::{unsigned::UnsignedCoins, Coins};

pub type DecimalCoins = Coins<Decimal256, DecimalCoin>;

impl DecimalCoins {
    /// Checks that all coins amount of the structure are bigger or equal to other coins.
    pub fn is_all_gte(&self, other: &[DecimalCoin]) -> bool {
        let other = other.iter().collect::<Vec<_>>();

        if other.is_empty() {
            return true;
        }

        for coin in other {
            // Note:
            // For all coins in other, self must contain a greater
            // than or equal amount of that coin. Thus if there exists
            // a single coin amount in other which is greater than the
            // self amount of the same coin we fail.
            if coin.amount > self.amount_of(&coin.denom) {
                return false;
            }
        }

        true
    }

    /// Adds matching coins amounts and creates new from unmatching coins.
    pub fn checked_add(&self, other: &DecimalCoins) -> Result<Self, CoinsError> {
        let coins = self.checked_calculate_iterate(other.inner(), Decimal256::checked_add)?;
        Self::new(coins)
    }

    fn checked_calculate_iterate(
        &self,
        other_coins: &[DecimalCoin],
        operation: impl Fn(Decimal256, Decimal256) -> Result<Decimal256, OverflowError>,
    ) -> Result<Vec<DecimalCoin>, CoinsError> {
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
                    result.push(DecimalCoin {
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

    /// Substracts matching coins. If the other coins have bigger values or the coins that don't
    /// exists in original set, method returns error. If all coins are identical method returns
    /// error.
    pub fn checked_sub(&self, other: &DecimalCoins) -> Result<Self, CoinsError> {
        if self.is_all_gte(other.inner()) {
            let coins: Vec<DecimalCoin> = self
                .checked_calculate_iterate(other.inner(), Decimal256::checked_sub)?
                .into_iter()
                // filter zeros after sub
                .filter(|c| !c.amount.is_zero())
                .collect();
            Self::new(coins)
        } else {
            Err(CoinsError::InvalidAmount)
        }
    }

    /// Multiplies each coin by a number and truncates decimal part from the result.
    pub fn checked_mul_dec_truncate(&self, multiplier: Decimal256) -> Result<Self, CoinsError> {
        let mut coins = vec![];
        for coin in self.inner().iter() {
            coins.push(DecimalCoin::new(
                coin.amount
                    .checked_mul(multiplier)
                    // TODO: extend error
                    .map_err(|_| CoinsError::InvalidAmount)?
                    .floor(),
                coin.denom.clone(),
            ));
        }

        Self::new(coins)
    }

    /// Multiplies each coin by a number and rounds decimal part for the result.
    pub fn checked_mul_dec(&self, multiplier: Decimal256) -> Result<Self, CoinsError> {
        let mut coins = vec![];
        for coin in self.inner().iter() {
            let normal = DecimalCoin::new(
                coin.amount
                    .checked_mul(multiplier)
                    .map_err(|_| CoinsError::InvalidAmount)?,
                coin.denom.clone(),
            );
            let mut floored = DecimalCoin::new(
                coin.amount
                    .checked_mul(multiplier)
                    .map_err(|_| CoinsError::InvalidAmount)?
                    .floor(),
                coin.denom.clone(),
            );

            if normal.amount - floored.amount
                >= Decimal256::from_atomics(5u64, 1).expect("hardcoded values cannot fail")
            {
                floored.amount += Decimal256::one();
            }
            coins.push(floored);
        }

        Self::new(coins)
    }

    /// Divides each coin by a number and truncates decimal part from the result.
    pub fn checked_quo_dec_truncate(&self, divider: Decimal256) -> Result<Self, CoinsError> {
        let mut coins = vec![];
        for coin in self.inner().iter() {
            coins.push(DecimalCoin::new(
                coin.amount
                    .checked_div(divider)
                    .map_err(|_| CoinsError::InvalidAmount)?
                    .floor(),
                coin.denom.clone(),
            ));
        }

        Self::new(coins)
    }

    /// split the coins into two parts: unsigned truncated coins and decimal change. Returns None for
    /// the part that contains only zeros.
    pub fn truncate_decimal(&self) -> (Option<UnsignedCoins>, Option<DecimalCoins>) {
        let (truncated, change): (Vec<UnsignedCoin>, Vec<DecimalCoin>) = self
            .storage
            .iter()
            .map(DecimalCoin::truncate_decimal)
            .unzip();

        let truncated: Vec<UnsignedCoin> = truncated
            .into_iter()
            .filter(|coin| !coin.amount.is_zero())
            .collect();
        let change: Vec<DecimalCoin> = change
            .into_iter()
            .filter(|coin| !coin.amount.is_zero())
            .collect();

        let truncated = if truncated.is_empty() {
            None
        } else {
            Some(
                UnsignedCoins::new(truncated)
                    .expect("inner structure of coins should be unchanged"),
            )
        };
        let change = if change.is_empty() {
            None
        } else {
            Some(DecimalCoins::new(change).expect("inner structure of coins should be unchanged"))
        };

        (truncated, change)
    }

    /// Intersect will return a new set of coins which contains the minimum DecCoin
    /// for common denoms found in both `coins` and `coinsB`. For denoms not common
    /// to both `coins` and `coinsB` the minimum is considered to be 0, thus they
    /// are not added to the final set. In other words, trim any denom amount from
    /// coin which exceeds that of coinB, such that (coin.Intersect(coinB)).IsLTE(coinB).
    /// See also Coins.Min().
    pub fn intersect(&self, other: &DecimalCoins) -> Result<Self, CoinsError> {
        let coins: Vec<_> = self
            .inner()
            .iter()
            .map(|coin| DecimalCoin {
                denom: coin.denom.clone(),
                amount: coin.amount.min(other.amount_of(&coin.denom)),
            })
            .filter(|coin| !coin.amount.is_zero())
            .collect();
        DecimalCoins::new(coins)
    }
}

impl TryFrom<Vec<UnsignedCoin>> for DecimalCoins {
    type Error = CoinsError;

    fn try_from(value: Vec<UnsignedCoin>) -> Result<Self, Self::Error> {
        // TODO: maybe add TryFrom<Coin> for DecimalCoin
        let mut dec_coins = vec![];
        for coin in value {
            let amount =
                Decimal256::from_atomics(coin.amount, 0).map_err(|_| CoinsError::InvalidAmount)?;
            dec_coins.push(DecimalCoin::new(amount, coin.denom));
        }
        Self::new(dec_coins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::denom::Denom;
    use cosmwasm_std::Uint256;
    use std::{str::FromStr, sync::OnceLock};

    static DENOMS: OnceLock<[Denom; 4]> = OnceLock::new();

    fn setup_denoms() {
        DENOMS.get_or_init(|| {
            [
                Denom::from_str("coin").expect("hardcoded value cannot fail"),
                Denom::from_str("stake").expect("hardcoded value cannot fail"),
                Denom::from_str("token").expect("hardcoded value cannot fail"),
                Denom::from_str("uatom").expect("hardcoded value cannot fail"),
            ]
        });
    }

    // don't restricted because of testing purpose
    fn generate_coins(amounts: Vec<u64>) -> DecimalCoins {
        setup_denoms();
        let mut coins = vec![];
        for (i, v) in amounts.into_iter().enumerate() {
            if v != 0 {
                coins.push(DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[i].clone(),
                    amount: Decimal256::from_atomics(v, 0).unwrap(),
                })
            }
        }
        DecimalCoins::new(coins).unwrap()
    }

    #[test]
    fn is_all_gte() -> anyhow::Result<()> {
        /* identical denoms */
        let dec_coins_1 = generate_coins(vec![100, 100]);
        let dec_coins_2 = generate_coins(vec![100, 50]);
        assert!(dec_coins_1.is_all_gte(dec_coins_1.inner()));
        assert!(dec_coins_1.is_all_gte(dec_coins_2.inner()));
        assert!(!dec_coins_2.is_all_gte(dec_coins_1.inner()));

        let dec_coins_1 = generate_coins(vec![100, 100]);
        let dec_coins_2 = generate_coins(vec![100, 150]);
        assert!(!dec_coins_1.is_all_gte(dec_coins_2.inner()));
        assert!(dec_coins_2.is_all_gte(dec_coins_1.inner()));

        /* not identical denoms */
        let dec_coins_1 = generate_coins(vec![100, 100, 100]);
        let dec_coins_2 = generate_coins(vec![50, 50, 0, 50]);
        // dec_coins_2 has not matching coins
        assert!(!dec_coins_1.is_all_gte(dec_coins_2.inner()));
        // dec_coins_1 has not matching coins and values are greater
        assert!(!dec_coins_2.is_all_gte(dec_coins_1.inner()));

        Ok(())
    }

    #[test]
    fn checked_add() -> anyhow::Result<()> {
        let dec_coins_1 = generate_coins(vec![100, 100, 100]);
        let dec_coins_2 = generate_coins(vec![50, 50, 0, 50]);
        let dec_coins_add = dec_coins_1.checked_add(&dec_coins_2).unwrap();
        assert_eq!(
            dec_coins_add.into_inner(),
            vec![
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    amount: Decimal256::from_atomics(150u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[1].clone(),
                    amount: Decimal256::from_atomics(150u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[2].clone(),
                    amount: Decimal256::from_atomics(100u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[3].clone(),
                    amount: Decimal256::from_atomics(50u64, 0).unwrap(),
                },
            ]
        );

        let dec_coins_inner = vec![DecimalCoin {
            denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
            amount: Decimal256::new(Uint256::MAX),
        }];
        let dec_coins_1 = DecimalCoins::new(dec_coins_inner.clone()).unwrap();
        let dec_coins_2 = DecimalCoins::new(dec_coins_inner).unwrap();

        assert!(dec_coins_1.checked_add(&dec_coins_2).is_err());

        Ok(())
    }

    #[test]
    fn checked_sub() -> anyhow::Result<()> {
        let dec_coins_1 = generate_coins(vec![100, 100, 100]);
        let dec_coins_2 = generate_coins(vec![50, 50, 0, 50]);
        // has not matching coins
        let dec_coins_sub = dec_coins_1.checked_sub(&dec_coins_2);
        assert!(dec_coins_sub.is_err());
        // removes all coins
        let dec_coins_sub = dec_coins_1.checked_sub(&dec_coins_1);
        assert!(dec_coins_sub.is_err());

        /* successful sub */
        let dec_coins_1 = generate_coins(vec![100, 100, 100]);
        let dec_coins_2 = generate_coins(vec![50, 50]);
        let dec_coins_sub = dec_coins_1.checked_sub(&dec_coins_2)?;
        assert_eq!(
            dec_coins_sub.inner(),
            &vec![
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    amount: Decimal256::from_atomics(50u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[1].clone(),
                    amount: Decimal256::from_atomics(50u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[2].clone(),
                    amount: Decimal256::from_atomics(100u64, 0).unwrap(),
                },
            ]
        );

        Ok(())
    }

    #[test]
    fn checked_mul_dec_truncate() -> anyhow::Result<()> {
        let dec_coins = generate_coins(vec![100, 90]);
        let dec_coins_mul_truncated = dec_coins.checked_mul_dec_truncate(
            Decimal256::from_atomics(31u64, 3).expect("hardcoded value can't fail"),
        )?;
        assert_eq!(
            dec_coins_mul_truncated.inner(),
            &vec![
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    // 3.03
                    amount: Decimal256::from_atomics(3u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[1].clone(),
                    // 2.73
                    amount: Decimal256::from_atomics(2u64, 0).unwrap(),
                },
            ]
        );

        let dec_coins_inner = vec![DecimalCoin {
            denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
            amount: Decimal256::new(Uint256::MAX),
        }];
        let dec_coins = DecimalCoins::new(dec_coins_inner.clone()).unwrap();
        let dec_coins_mul_truncated =
            dec_coins.checked_mul_dec_truncate(Decimal256::new(Uint256::MAX));
        assert!(dec_coins_mul_truncated.is_err());

        Ok(())
    }

    #[test]
    fn checked_mul_dec() -> anyhow::Result<()> {
        let dec_coins = generate_coins(vec![110, 100, 90]);
        let dec_coins_mul = dec_coins.checked_mul_dec(
            Decimal256::from_atomics(25u64, 3).expect("hardcoded value can't fail"),
        )?;
        assert_eq!(
            dec_coins_mul.inner(),
            &vec![
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    // > 2.5
                    amount: Decimal256::from_atomics(3u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[1].clone(),
                    // == 2.5
                    amount: Decimal256::from_atomics(3u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[2].clone(),
                    // < 2.5
                    amount: Decimal256::from_atomics(2u64, 0).unwrap(),
                },
            ]
        );

        let dec_coins_inner = vec![DecimalCoin {
            denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
            amount: Decimal256::new(Uint256::MAX),
        }];
        let dec_coins = DecimalCoins::new(dec_coins_inner.clone()).unwrap();
        let dec_coins_mul = dec_coins.checked_mul_dec(Decimal256::new(Uint256::MAX));
        assert!(dec_coins_mul.is_err());

        Ok(())
    }

    #[test]
    fn checked_quo_dec_truncate() -> anyhow::Result<()> {
        let dec_coins = generate_coins(vec![17, 12]);
        let dec_coins_quo_truncated = dec_coins.checked_quo_dec_truncate(
            Decimal256::from_atomics(10u64, 0).expect("hardcoded value can't fail"),
        )?;
        assert_eq!(
            dec_coins_quo_truncated.inner(),
            &vec![
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    // 1.7
                    amount: Decimal256::from_atomics(1u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[1].clone(),
                    // 1.2
                    amount: Decimal256::from_atomics(1u64, 0).unwrap(),
                },
            ]
        );

        let dec_coins = generate_coins(vec![1]);
        let dec_coins_quo_truncated =
            dec_coins.checked_quo_dec_truncate(Decimal256::new(Uint256::from(0u64)));
        assert!(dec_coins_quo_truncated.is_err());

        Ok(())
    }

    #[test]
    fn truncate_decimal() -> anyhow::Result<()> {
        let dec_coins = generate_coins(vec![17]);
        let (truncated, change) = dec_coins.truncate_decimal();
        assert_eq!(
            truncated,
            Some(
                UnsignedCoins::new(vec![UnsignedCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    amount: Uint256::from(17u64),
                },])
                .unwrap()
            )
        );
        assert!(change.is_none());

        let dec_coins_inner = vec![DecimalCoin {
            denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
            amount: Decimal256::from_atomics(5u64, 1).expect("hardcoded value cannot fail"),
        }];
        let dec_coins = DecimalCoins::new(dec_coins_inner.clone()).unwrap();
        let (truncated, change) = dec_coins.truncate_decimal();
        assert!(truncated.is_none());
        assert_eq!(
            change,
            Some(
                DecimalCoins::new(vec![DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    amount: Decimal256::from_atomics(5u64, 1).unwrap(),
                },])
                .unwrap()
            )
        );

        let dec_coins_inner = vec![DecimalCoin {
            denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
            amount: Decimal256::from_atomics(175u64, 1).expect("hardcoded value cannot fail"),
        }];
        let dec_coins = DecimalCoins::new(dec_coins_inner.clone()).unwrap();
        let (truncated, change) = dec_coins.truncate_decimal();
        assert_eq!(
            truncated,
            Some(
                UnsignedCoins::new(vec![UnsignedCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    amount: Uint256::from(17u64),
                },])
                .unwrap()
            )
        );
        assert_eq!(
            change,
            Some(
                DecimalCoins::new(vec![DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    amount: Decimal256::from_atomics(5u64, 1).unwrap(),
                },])
                .unwrap()
            )
        );

        Ok(())
    }

    #[test]
    fn intersects() -> anyhow::Result<()> {
        /* has intersections */
        let dec_coins_1 = generate_coins(vec![90, 0, 100, 30]);
        let dec_coins_2 = generate_coins(vec![100, 50, 20]);
        let dec_intersects = dec_coins_1.intersect(&dec_coins_2)?;
        assert_eq!(
            dec_intersects.inner(),
            &vec![
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[0].clone(),
                    amount: Decimal256::from_atomics(90u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: DENOMS.get().expect("cannot fail initialized variable")[2].clone(),
                    // 1.2
                    amount: Decimal256::from_atomics(20u64, 0).unwrap(),
                },
            ]
        );

        /* don't have intersections */
        let dec_coins_1 = generate_coins(vec![90, 0]);
        let dec_coins_2 = generate_coins(vec![0, 50]);
        let dec_intersects = dec_coins_1.intersect(&dec_coins_2);
        assert!(dec_intersects.is_err());

        Ok(())
    }
}
