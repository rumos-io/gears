use auth::{AuthABCIHandler, GenesisState, Keeper};
use gears::{
    derive::{ParamsKeys, StoreKeys},
    tendermint::types::time::timestamp::Timestamp,
    utils::node::{init_node, GenesisSource, MockOptionsFormer},
};

use gears::{types::address::AccAddress, x::module::Module};

#[test]
/// In this scenario, we test the initialization of the application and execute a few blocks
fn test_init_and_few_blocks() {
    let opt: MockOptionsFormer<
        SubspaceKey,
        AuthABCIHandler<SpaceKey, SubspaceKey, AuthModules>,
        GenesisState,
    > = MockOptionsFormer::new()
        .abci_handler(AuthABCIHandler::new(Keeper::new(
            SpaceKey::Auth,
            SubspaceKey::Auth,
            AuthModules::FeeCollector,
        )))
        .baseapp_sbs_key(SubspaceKey::BaseApp)
        .genesis(GenesisSource::Genesis(GenesisState::default()));

    let mut node = init_node(opt);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "8d3663f81a98bec58a6d3a9f39c38469438bfecd8257dd335c8c047b933b08ad"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "8d3663f81a98bec58a6d3a9f39c38469438bfecd8257dd335c8c047b933b08ad"
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthModules {
    FeeCollector,
}

impl Module for AuthModules {
    fn name(&self) -> String {
        match self {
            AuthModules::FeeCollector => "fee_collector".into(),
        }
    }

    fn address(&self) -> AccAddress {
        match self {
            AuthModules::FeeCollector => {
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
        }
    }

    fn permissions(&self) -> Vec<String> {
        match self {
            AuthModules::FeeCollector => Vec::new(),
        }
    }
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, StoreKeys)]
#[skey(params = Params)]
pub enum SpaceKey {
    #[skey(to_string = "acc")]
    Auth,
    #[skey(to_string = "params")]
    Params,
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys)]
pub enum SubspaceKey {
    #[pkey(to_string = "auth/")]
    Auth,
    #[pkey(to_string = "baseapp/")]
    BaseApp,
}
