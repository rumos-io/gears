use auth::{AuthABCIHandler, GenesisState, Keeper};
use gears::{
    tendermint::types::time::timestamp::Timestamp,
    utils::node::{init_node, GenesisSource, MockOptionsFormer},
};
use std::str::FromStr;

use gears::{
    params::{ParamsSubspaceKey, SubspaceParseError},
    store::StoreKey,
    types::address::AccAddress,
    x::module::Module,
};

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

    let (mut node, _) = init_node(opt);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "8d3663f81a98bec58a6d3a9f39c38469438bfecd8257dd335c8c047b933b08ad"
    );

    for _ in 0..100 {
        node.step(vec![], Timestamp::UNIX_EPOCH);
    }

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
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
    fn get_name(&self) -> String {
        match self {
            AuthModules::FeeCollector => "fee_collector".into(),
        }
    }

    fn get_address(&self) -> AccAddress {
        match self {
            AuthModules::FeeCollector => {
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
        }
    }

    fn get_permissions(&self) -> Vec<String> {
        match self {
            AuthModules::FeeCollector => vec![],
        }
    }
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum SpaceKey {
    Auth,
    Params,
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum SubspaceKey {
    Auth,
    BaseApp,
}

impl FromStr for SubspaceKey {
    type Err = SubspaceParseError;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err(SubspaceParseError("omit".to_string()))
    }
}

impl FromStr for SpaceKey {
    type Err = SubspaceParseError;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err(SubspaceParseError("omit".to_string()))
    }
}

impl ParamsSubspaceKey for SubspaceKey {
    fn name(&self) -> &'static str {
        match self {
            SubspaceKey::BaseApp => "baseapp/",
            SubspaceKey::Auth => "auth/",
        }
    }
}

impl StoreKey for SpaceKey {
    fn name(&self) -> &'static str {
        match self {
            SpaceKey::Auth => "acc",
            SpaceKey::Params => "params",
        }
    }

    fn params() -> &'static Self {
        const PARAM_KEY: SpaceKey = SpaceKey::Params;

        &PARAM_KEY
    }
}
