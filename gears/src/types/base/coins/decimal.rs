use std::cmp::Ordering;

use cosmwasm_std::{Decimal256, OverflowError};

use crate::types::base::{
    coin::{DecimalCoin, UnsignedCoin},
    errors::CoinsError,
};

use super::{unsigned::UnsignedCoins, Coins};

pub type DecimalCoins = Coins<Decimal256, DecimalCoin>;

impl DecimalCoins {
    pub fn is_all_gte(&self, other: &[DecimalCoin]) -> bool {
        let other = other.iter().collect::<Vec<_>>();

        if other.is_empty() {
            return true;
        }

        for coin in other {
            if coin.amount > self.amount_of(&coin.denom) {
                return false;
            }
        }

        true
    }

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

    pub fn checked_sub(&self, other: &DecimalCoins) -> Result<Self, CoinsError> {
        if self.is_all_gte(other.inner()) {
            let coins = self.checked_calculate_iterate(other.inner(), Decimal256::checked_sub)?;
            Self::new(coins)
        } else {
            Err(CoinsError::InvalidAmount)
        }
    }

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

    pub fn checked_mul_dec(&self, multiplier: Decimal256) -> Result<Self, CoinsError> {
        let mut coins = vec![];
        for coin in self.inner().iter() {
            let normal = DecimalCoin::new(
                coin.amount
                    .checked_mul(multiplier)
                    .map_err(|_| CoinsError::InvalidAmount)?
                    .floor(),
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

    pub fn truncate_decimal(&self) -> (UnsignedCoins, DecimalCoins) {
        let (truncated, change): (Vec<UnsignedCoin>, Vec<DecimalCoin>) = self
            .storage
            .iter()
            .map(DecimalCoin::truncate_decimal)
            .unzip();

        (
            UnsignedCoins::new(truncated.into_iter().filter(|c| !c.amount.is_zero()))
                .expect("inner structure of coins should be unchanged"),
            DecimalCoins::new(change.into_iter().filter(|c| !c.amount.is_zero()))
                .expect("inner structure of coins should be unchanged"),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::types::denom::Denom;

    use super::*;

    #[test]
    fn checked_add_and_sub() -> anyhow::Result<()> {
        let denom_1 = Denom::from_str("uatom").expect("hardcoded value cannot fail");
        let denom_2 = Denom::from_str("stake").expect("hardcoded value cannot fail");
        let denom_3 = Denom::from_str("coin").expect("hardcoded value cannot fail");
        let denom_4 = Denom::from_str("token").expect("hardcoded value cannot fail");

        let dec_coins_inner_1 = vec![
            DecimalCoin {
                denom: denom_3.clone(),
                amount: Decimal256::from_atomics(100u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_2.clone(),
                amount: Decimal256::from_atomics(100u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_1.clone(),
                amount: Decimal256::from_atomics(100u64, 0).unwrap(),
            },
        ];
        let dec_coins_1 = DecimalCoins::new(dec_coins_inner_1).unwrap();
        let dec_coins_inner_2 = vec![
            DecimalCoin {
                denom: denom_3.clone(),
                amount: Decimal256::from_atomics(50u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_2.clone(),
                amount: Decimal256::from_atomics(50u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_4.clone(),
                amount: Decimal256::from_atomics(50u64, 0).unwrap(),
            },
        ];
        let dec_coins_2 = DecimalCoins::new(dec_coins_inner_2.clone()).unwrap();

        /* test checked_add */
        let dec_coins_add = dec_coins_1.checked_add(&dec_coins_2).unwrap();
        assert_eq!(
            dec_coins_add.into_inner(),
            vec![
                DecimalCoin {
                    denom: denom_3.clone(),
                    amount: Decimal256::from_atomics(150u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: denom_2.clone(),
                    amount: Decimal256::from_atomics(150u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: denom_4.clone(),
                    amount: Decimal256::from_atomics(50u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: denom_1.clone(),
                    amount: Decimal256::from_atomics(100u64, 0).unwrap(),
                },
            ]
        );

        /* test checked_sub */
        let dec_coins_sub = dec_coins_1.checked_sub(&dec_coins_2);
        assert!(dec_coins_sub.is_err());

        /* successful sub */
        let dec_coins_2 = DecimalCoins::new(dec_coins_inner_2[0..2].to_vec()).unwrap();
        let dec_coins_sub = dec_coins_1.checked_sub(&dec_coins_2)?;
        assert_eq!(
            dec_coins_sub.inner(),
            &vec![
                DecimalCoin {
                    denom: denom_3.clone(),
                    amount: Decimal256::from_atomics(50u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: denom_2.clone(),
                    amount: Decimal256::from_atomics(50u64, 0).unwrap(),
                },
                DecimalCoin {
                    denom: denom_1.clone(),
                    amount: Decimal256::from_atomics(100u64, 0).unwrap(),
                },
            ]
        );

        Ok(())
    }

    #[test]
    fn is_all_gte() -> anyhow::Result<()> {
        let denom_1 = Denom::from_str("uatom").expect("hardcoded value cannot fail");
        let denom_2 = Denom::from_str("stake").expect("hardcoded value cannot fail");
        let denom_3 = Denom::from_str("coin").expect("hardcoded value cannot fail");
        let denom_4 = Denom::from_str("token").expect("hardcoded value cannot fail");

        let dec_coins_1 = DecimalCoins::new(vec![
            DecimalCoin {
                denom: denom_3.clone(),
                amount: Decimal256::from_atomics(100u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_2.clone(),
                amount: Decimal256::from_atomics(100u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_1.clone(),
                amount: Decimal256::from_atomics(100u64, 0).unwrap(),
            },
        ])
        .unwrap();
        let dec_coins_2 = DecimalCoins::new(vec![
            DecimalCoin {
                denom: denom_3.clone(),
                amount: Decimal256::from_atomics(50u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_2.clone(),
                amount: Decimal256::from_atomics(50u64, 0).unwrap(),
            },
            DecimalCoin {
                denom: denom_4.clone(),
                amount: Decimal256::from_atomics(50u64, 0).unwrap(),
            },
        ])
        .unwrap();

        assert!(dec_coins_1.is_all_gte(dec_coins_1.inner()));
        assert!(!dec_coins_1.is_all_gte(dec_coins_2.inner()));
        assert!(!dec_coins_2.is_all_gte(dec_coins_1.inner()));
        Ok(())
    }
}
