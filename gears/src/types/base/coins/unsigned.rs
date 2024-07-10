use cosmwasm_std::Uint256;

use crate::types::base::coin::Coin;

use super::Coins;

pub type UnsignedCoins = Coins<Uint256, Coin>;

impl UnsignedCoins {
    pub fn is_all_gte<'a>(&self, other: impl IntoIterator<Item = &'a Coin>) -> bool {
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
