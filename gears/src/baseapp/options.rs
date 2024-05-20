use std::sync::{Arc, RwLock};

use crate::{error::POISONED_LOCK, types::base::decimal_coin::DecimalCoin};

#[derive(Debug, Clone, Default)]
pub struct NodeOptions(Arc<RwLock<InnerOptions>>);

#[derive(Debug, Default)]
struct InnerOptions {
    pub min_gas_prices: Vec<DecimalCoin>,
}

impl NodeOptions {
    pub fn new(min_gas_prices: Vec<DecimalCoin>) -> Self {
        Self(Arc::new(RwLock::new(InnerOptions { min_gas_prices })))
    }

    pub fn min_gas_prices(&self) -> Vec<DecimalCoin> {
        self.0
            .read()
            .expect(POISONED_LOCK)
            .min_gas_prices
            .to_owned()
    }
}
