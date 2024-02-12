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
pub(super) mod test_mocks {
    //! This module shares test implementation of context and `StoreKey`
    //!
    use database::{Database, PrefixDB};
    use gears::types::context::context::ContextTrait;
    use proto_messages::cosmos::tx::v1beta1::tx_metadata::{DenomUnit, Metadata};
    use store::StoreKey;
    use strum::EnumIter;

    // We use custom implementation instead of mock
    // 1. Mockall requires generic parameters to be 'static
    // 2. Diffuclties exporting mock on other crates
    pub struct MockContext;

    impl<T: Database, SK: StoreKey> ContextTrait<T, SK> for MockContext {
        fn height(&self) -> u64 {
            unimplemented!()
        }

        fn chain_id(&self) -> &str {
            unimplemented!()
        }

        fn push_event(&mut self, _: tendermint::informal::abci::Event) {
            unimplemented!()
        }

        fn append_events(&mut self, _: Vec<tendermint::informal::abci::Event>) {
            unimplemented!()
        }

        fn metadata_get(&self) -> Metadata {
            Metadata {
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
                uri: String::new(),
                uri_hash: None,
            }
        }
        fn get_kv_store(&self, _store_key: &SK) -> &store::KVStore<PrefixDB<T>> {
            unimplemented!()
        }

        fn get_mutable_kv_store(&mut self, _store_key: &SK) -> &mut store::KVStore<PrefixDB<T>> {
            unimplemented!()
        }
    }

    #[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
    pub enum KeyMock {
        Bank,
        Auth,
        Params,
    }

    impl StoreKey for KeyMock {
        fn name(&self) -> &'static str {
            match self {
                KeyMock::Bank => "bank",
                KeyMock::Auth => "acc",
                KeyMock::Params => "params",
            }
        }
    }
}
