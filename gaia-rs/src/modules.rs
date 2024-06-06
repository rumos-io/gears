use gears::types::address::AccAddress;
use gears::x::module::{Module, ModuleKey};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GaiaModuleKey {
    FeeCollector,
}

impl ModuleKey for GaiaModuleKey {
    fn key(&self) -> &str {
        match self {
            GaiaModuleKey::FeeCollector => "fee_collector",
        }
    }
}

pub fn modules_map() -> HashMap<GaiaModuleKey, Module> {
    //TODO: construct address from Vec<u8> + make address constant
    //TODO: where do these addresses come from?
    let fee_collector_address =
        AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
            .expect("hard coded address is valid");
    let fee_collector_module = Module::new(
        GaiaModuleKey::FeeCollector.key().to_string(),
        fee_collector_address,
        vec![],
    );
    let mut res = HashMap::with_capacity(1);
    res.insert(GaiaModuleKey::FeeCollector, fee_collector_module);
    res
}
