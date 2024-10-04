use std::str::FromStr;

use anyhow::Ok;
use gears::x::module::Module;

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumIter)]
pub enum GaiaModules {
    FeeCollector,
    BondedPool,
    NotBondedPool,
}

impl Module for GaiaModules {
    fn name(&self) -> String {
        match self {
            GaiaModules::FeeCollector => "fee_collector".into(),
            GaiaModules::BondedPool => staking::BONDED_POOL_NAME.into(),
            GaiaModules::NotBondedPool => staking::NOT_BONDED_POOL_NAME.into(),
        }
    }

    fn permissions(&self) -> Vec<String> {
        match self {
            GaiaModules::FeeCollector => vec![],
            GaiaModules::BondedPool => vec!["burner".into(), "staking".into()],
            GaiaModules::NotBondedPool => vec!["burner".into(), "staking".into()],
        }
    }
}

impl FromStr for GaiaModules {
    type Err = anyhow::Error; // TODO: proper error

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "fee_collector" => Self::FeeCollector,
            staking::BONDED_POOL_NAME => Self::BondedPool,
            staking::NOT_BONDED_POOL_NAME => Self::NotBondedPool,
            _ => Err(anyhow::anyhow!("Failed to parse modules"))?,
        })
    }
}
