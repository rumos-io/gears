use std::sync::{Arc, RwLock};

use crate::{error::POISONED_LOCK, types::base::min_gas::MinGasPrices};

#[derive(Debug, Clone, Default)]
pub struct NodeOptions(Arc<RwLock<InnerOptions>>);

#[derive(Debug, Default)]
struct InnerOptions {
    pub min_gas_prices: MinGasPrices,
}

impl NodeOptions {
    pub fn new(min_gas_prices: MinGasPrices) -> Self {
        Self(Arc::new(RwLock::new(InnerOptions { min_gas_prices })))
    }

    pub fn min_gas_prices(&self) -> MinGasPrices {
        self.0
            .read()
            .expect(POISONED_LOCK)
            .min_gas_prices
            .to_owned()
    }
}
