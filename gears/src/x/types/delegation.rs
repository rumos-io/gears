use cosmwasm_std::Decimal256;

use crate::types::address::{AccAddress, ValAddress};

pub trait StakingDelegation {
    fn delegator(&self) -> &AccAddress;
    fn validator(&self) -> &ValAddress;
    fn shares(&self) -> &Decimal256;
}
