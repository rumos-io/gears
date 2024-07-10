use crate::types::base::coin::{Coin, DecimalCoin};
use cosmwasm_std::Decimal256;

use super::{unsigned::UnsignedCoins, Coins};

pub type DecimalCoins = Coins<Decimal256, DecimalCoin>;

impl DecimalCoins {
    pub fn is_all_gte<'a>(&self, other: impl IntoIterator<Item = &'a DecimalCoin>) -> bool {
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

    pub fn truncate_decimal(&self) -> (UnsignedCoins, DecimalCoins) {
        let (truncated, change): (Vec<Coin>, Vec<DecimalCoin>) = self
            .storage
            .iter()
            .map(|(denom, amount)| DecimalCoin {
                denom: denom.clone(),
                amount: amount.clone(),
            })
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
