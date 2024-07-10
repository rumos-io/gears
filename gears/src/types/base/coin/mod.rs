mod decimal;
mod unsigned;

pub use decimal::*;
pub use unsigned::*;

use crate::types::denom::Denom;

pub trait Coin<T>: Clone {
    fn denom(&self) -> &Denom;
    fn amount(&self) -> &T;
}
