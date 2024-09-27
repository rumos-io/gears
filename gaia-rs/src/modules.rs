use gears::x::module::Module;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn get_permissions(&self) -> Vec<String> {
        match self {
            GaiaModules::FeeCollector => vec![],
            GaiaModules::BondedPool => vec!["burner".into(), "staking".into()],
            GaiaModules::NotBondedPool => vec!["burner".into(), "staking".into()],
        }
    }
}
