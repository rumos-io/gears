use gears::ibc::address::AccAddress;

pub enum Module {
    FeeCollector,
}

// //TODO: use properly typed QueryAccountResponse and QueryAccountRequest

impl Module {
    pub fn get_address(&self) -> AccAddress {
        match self {
            Module::FeeCollector => {
                //TODO: construct address from Vec<u8> + make address constant
                //TODO: where do these addresses come from?
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Module::FeeCollector => "fee_collector".into(),
        }
    }

    pub fn get_permissions(&self) -> Vec<String> {
        match self {
            Module::FeeCollector => vec![],
        }
    }
}
