mod decimal;
mod unsigned;

pub use decimal::*;
pub use unsigned::*;

use crate::types::denom::Denom;

pub trait Coin: Clone {
    type Amount;

    fn denom(&self) -> &Denom;
    fn amount(&self) -> &Self::Amount;
}
