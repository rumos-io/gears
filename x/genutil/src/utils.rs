use std::{collections::HashMap, path::Path};

use gears::types::{account::Account, address::AccAddress, base::coins::UnsignedCoins};
use staking::StakingParams;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GenesisAccounts {
    accounts: HashMap<AccAddress, Account>,
}

impl GenesisAccounts {
    pub fn new(sk: &str, genesis_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(genesis_path)?)?;

        let value = value
            .get_mut("app_state")
            .ok_or(anyhow::anyhow!("missing `app_state`"))?
            .get_mut(sk)
            .ok_or(anyhow::anyhow!("auth module {sk} is not found"))?
            .get_mut("accounts")
            .ok_or(anyhow::anyhow!("accounts is not found"))?
            .take();

        let values: Vec<Account> = serde_json::from_value(value)?;

        Ok(Self {
            accounts: values
                .into_iter()
                .map(|this| (this.get_address().clone(), this))
                .collect(),
        })
    }

    pub fn into_inner(self) -> HashMap<AccAddress, Account> {
        self.accounts
    }
}

impl IntoIterator for GenesisAccounts {
    type Item = (AccAddress, Account);

    type IntoIter = ::std::collections::hash_map::IntoIter<AccAddress, Account>;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.into_iter()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GenesisBalance {
    pub address: AccAddress,
    pub coins: UnsignedCoins,
}

#[derive(Debug, Clone)]
pub struct GenesisBalanceIter(HashMap<AccAddress, UnsignedCoins>);

impl GenesisBalanceIter {
    pub fn new(sk: &str, genesis_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(genesis_path)?)?;

        let value = value
            .get_mut("app_state")
            .ok_or(anyhow::anyhow!("missing `app_state`"))?
            .get_mut(sk)
            .ok_or(anyhow::anyhow!("bank module {sk} is not found"))?
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

    pub fn into_inner(self) -> HashMap<AccAddress, UnsignedCoins> {
        self.0
    }

    pub fn into_vec(self) -> Vec<GenesisBalance> {
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

pub fn parse_staking_params_from_genesis(
    sk: &str,
    params_str: &str,
    genesis_path: impl AsRef<Path>,
) -> anyhow::Result<StakingParams> {
    let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(genesis_path)?)?;

    let value = value
        .get_mut("app_state")
        .ok_or(anyhow::anyhow!("missing `app_state`"))?
        .get_mut(sk)
        .ok_or(anyhow::anyhow!("staking module is not found"))?
        .get_mut(params_str)
        .ok_or(anyhow::anyhow!("params is not found"))?
        .take();

    let result = serde_json::from_value::<StakingParams>(value)?;

    Ok(result)
}
