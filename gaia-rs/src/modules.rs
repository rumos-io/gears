use gears::types::address::AccAddress;
use gears::x::module::Module;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GaiaModules {
    FeeCollector,
}

impl Module for GaiaModules {
    fn get_name(&self) -> String {
        match self {
            GaiaModules::FeeCollector => "fee_collector".into(),
        }
    }

    fn get_address(&self) -> AccAddress {
        match self {
            GaiaModules::FeeCollector => {
                //TODO: construct address from Vec<u8> + make address constant
                //TODO: where do these addresses come from?
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
        }
    }

    fn get_permissions(&self) -> Vec<String> {
        match self {
            GaiaModules::FeeCollector => vec![],
        }
    }
}
