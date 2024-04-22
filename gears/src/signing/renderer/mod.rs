pub mod messages;
pub mod primitives;
pub mod tx;
pub mod value_renderer;

#[cfg(test)]
pub(super) mod test_functions {
    use crate::{
        signing::handler::MetadataGetter,
        types::{
            denom::Denom,
            tx::metadata::{DenomUnit, Metadata},
        },
    };

    pub struct TestNoneMetadataGetter;

    impl MetadataGetter for TestNoneMetadataGetter {
        type Error = std::io::Error; // this is not used here

        fn metadata(&self, _denom: &Denom) -> Result<Option<Metadata>, Self::Error> {
            Ok(None)
        }
    }

    pub struct TestMetadataGetter;

    impl MetadataGetter for TestMetadataGetter {
        type Error = std::io::Error; // this is not used here

        fn metadata(&self, denom: &Denom) -> Result<Option<Metadata>, Self::Error> {
            match denom.to_string().as_str() {
                "uatom" => Ok(Some(Metadata {
                    description: String::new(),
                    denom_units: vec![
                        DenomUnit {
                            denom: "ATOM".parse().expect("this is a valid denom"),
                            exponent: 6,
                            aliases: Vec::new(),
                        },
                        DenomUnit {
                            denom: "uatom".parse().expect("this is a valid denom"),
                            exponent: 0,
                            aliases: Vec::new(),
                        },
                    ],
                    base: "uatom".into(),
                    display: "ATOM".into(),
                    name: String::new(),
                    symbol: String::new(),
                })),
                "uon" => Ok(Some(Metadata {
                    description: String::new(),
                    denom_units: vec![
                        DenomUnit {
                            denom: "AAUON".parse().expect("this is a valid denom"),
                            exponent: 6,
                            aliases: Vec::new(),
                        },
                        DenomUnit {
                            denom: "uon".parse().expect("this is a valid denom"),
                            exponent: 0,
                            aliases: Vec::new(),
                        },
                    ],
                    base: "uon".into(),
                    display: "AAUON".into(),
                    name: String::new(),
                    symbol: String::new(),
                })),
                _ => Ok(None),
            }
        }
    }
}
