use cosmwasm_std::{Decimal256, Uint256};

pub mod coin;
pub mod coins;
pub mod errors;
pub mod min_gas;

pub trait ZeroNumeric: Clone {
    fn is_zero(&self) -> bool;

    fn zero() -> Self;
    fn one() -> Self;
}

impl ZeroNumeric for Uint256 {
    fn is_zero(&self) -> bool {
        self.is_zero()
    }

    fn zero() -> Self {
        Self::zero()
    }

    fn one() -> Self {
        Self::one()
    }
}

impl ZeroNumeric for Decimal256 {
    fn is_zero(&self) -> bool {
        self.is_zero()
    }

    fn zero() -> Self {
        Self::zero()
    }

    fn one() -> Self {
        Self::one()
    }
}
