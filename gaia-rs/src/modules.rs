use gears::x::module::Module;

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumIter)]
pub enum GaiaModules {
    FeeCollector,
    BondedPool,
    NotBondedPool,
    Gov,
}

impl Module for GaiaModules {
    fn name(&self) -> String {
        match self {
            GaiaModules::FeeCollector => "fee_collector".into(),
            GaiaModules::BondedPool => staking::BONDED_POOL_NAME.into(),
            GaiaModules::NotBondedPool => staking::NOT_BONDED_POOL_NAME.into(),
            GaiaModules::Gov => "gov".into(),
        }
    }

    fn permissions(&self) -> Vec<String> {
        match self {
            GaiaModules::FeeCollector => Vec::new(),
            GaiaModules::BondedPool => vec!["burner".into(), "staking".into()],
            GaiaModules::NotBondedPool => vec!["burner".into(), "staking".into()],
            GaiaModules::Gov => vec!["burner".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumIter, Hash, PartialOrd, Ord)]
pub enum GaiaXmodules {
    Auth,
    Bank,
    Staking,
}

impl TryFrom<Vec<u8>> for GaiaXmodules {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let val = match String::from_utf8(value) {
            Ok(val) => match val.as_str() {
                "auth" => GaiaXmodules::Auth,
                "bank" => GaiaXmodules::Bank,
                "staking" => GaiaXmodules::Staking,
                _ => Err(anyhow::anyhow!("no such module exists"))?,
            },
            Err(_) => Err(anyhow::anyhow!("invalid string"))?,
        };

        Ok(val)
    }
}

impl upgrade::Module for GaiaXmodules {
    fn name(&self) -> &'static str {
        match self {
            GaiaXmodules::Auth => "auth",
            GaiaXmodules::Bank => "bank",
            GaiaXmodules::Staking => "staking",
        }
    }
}
