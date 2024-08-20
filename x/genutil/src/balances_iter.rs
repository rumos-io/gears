use std::{collections::HashMap, path::Path};

use gears::{
    store::StoreKey,
    types::{address::AccAddress, base::coins::UnsignedCoins},
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GenesisBalance {
    pub address: AccAddress,
    pub coins: UnsignedCoins,
}

#[derive(Debug, Clone)]
pub struct GenesisBalanceIter(HashMap<AccAddress, UnsignedCoins>);

impl GenesisBalanceIter {
    pub fn new<SK: StoreKey>(sk: &SK, genesis_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(genesis_path)?)?;

        let value = value
            .get_mut(sk.name())
            .ok_or(anyhow::anyhow!("module is not found"))?
            .get_mut("balances")
            .ok_or(anyhow::anyhow!("balances is not found"))?
            .take();

        let values: Vec<GenesisBalance> = serde_json::from_value(value)?;

        Ok(Self(
            values
                .into_iter()
                .map(|GenesisBalance { address, coins }| (address, coins))
                .collect(),
        ))
    }

    pub fn into_inner(self) -> Vec<GenesisBalance> {
        self.0
            .into_iter()
            .map(|(address, coins)| GenesisBalance { address, coins })
            .collect()
    }
}

impl IntoIterator for GenesisBalanceIter {
    type Item = (AccAddress, UnsignedCoins);

    type IntoIter = ::std::collections::hash_map::IntoIter<AccAddress, UnsignedCoins>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
