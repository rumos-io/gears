pub mod auth_info;
pub mod coin;
pub mod fee;
pub mod mode_info;
pub mod public_key_enum;
pub mod secp256_pubkey;
pub mod send_coins;
pub mod signer_data;
pub mod signer_info;
pub mod textual_data;
pub mod tip;

#[cfg(test)]
pub(super) mod test_functions {
    use proto_messages::cosmos::tx::v1beta1::tx_metadata::{DenomUnit, Metadata};
    use proto_types::Denom;

    pub fn get_metadata(denom: &Denom) -> Option<Metadata> {
        match denom.to_string().as_str() {
            "uatom" => Some(Metadata {
                description: String::new(),
                denom_units: vec![
                    DenomUnit {
                        denom: "ATOM".parse().expect("Test data should be valid"),
                        exponent: 6,
                        aliases: Vec::new(),
                    },
                    DenomUnit {
                        denom: "uatom".parse().expect("Test data should be valid"),
                        exponent: 0,
                        aliases: Vec::new(),
                    },
                ],
                base: "uatom".into(),
                display: "ATOM".into(),
                name: String::new(),
                symbol: String::new(),
            }),
            "uon" => Some(Metadata {
                description: String::new(),
                denom_units: vec![
                    DenomUnit {
                        denom: "UON".parse().expect("Test data should be valid"),
                        exponent: 6,
                        aliases: Vec::new(),
                    },
                    DenomUnit {
                        denom: "uon".parse().expect("Test data should be valid"),
                        exponent: 0,
                        aliases: Vec::new(),
                    },
                ],
                base: "uon".into(),
                display: "UON".into(),
                name: String::new(),
                symbol: String::new(),
            }),
            _ => None,
        }
    }
}
