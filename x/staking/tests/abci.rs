use std::str::FromStr;

use gears::{
    application::handlers::node::ModuleInfo,
    derive::{ParamsKeys, StoreKeys},
    tendermint::types::time::timestamp::Timestamp,
    types::{address::AccAddress, base::coin::UnsignedCoin},
    utils::node::{init_node, GenesisSource, MockOptionsFormer},
    x::{
        keepers::mocks::{auth::MockAuthKeeper, bank::MockBankKeeper},
        module::Module,
    },
};
use staking::{GenesisState, Keeper, MockHookKeeper, StakingABCIHandler};

#[test]
/// In this scenario, we test the initialization of the application and execute a few blocks
fn test_init_and_few_blocks() {
    let opt: MockOptionsFormer<
        SubspaceKey,
        StakingABCIHandler<
            SpaceKey,
            SubspaceKey,
            MockAuthKeeper,
            MockBankKeeper,
            MockHookKeeper<SpaceKey, MockAuthKeeper, StakingModules>,
            StakingModules,
            BankModuleInfo,
        >,
        GenesisState,
    > = MockOptionsFormer::new()
        .abci_handler(StakingABCIHandler::new(Keeper::new(
            SpaceKey::Auth,
            SubspaceKey::Auth,
            MockAuthKeeper::former().form(),
            MockBankKeeper::former()
                .balance(UnsignedCoin::from_str("34uaton").expect("valid default"))
                .form(),
            None,
            StakingModules::BondedPool,
            StakingModules::NotBondedPool,
        )))
        .baseapp_sbs_key(SubspaceKey::BaseApp)
        .genesis(GenesisSource::Default);

    let (mut node, _) = init_node(opt);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "67647df38f8fe610ef4c15581f73ac76d4c8598f02db3fb3cf23052a9de7da22"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "0b82cabbbe14529b18ce923a4599cfa8ce5b9557fc8bf3d9a430af4858de3632"
    );
}

#[derive(Debug, Clone)]
struct BankModuleInfo;

impl ModuleInfo for BankModuleInfo {
    const NAME: &'static str = "bank";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakingModules {
    FeeCollector,
    BondedPool,
    NotBondedPool,
}

impl Module for StakingModules {
    fn name(&self) -> String {
        match self {
            StakingModules::FeeCollector => "fee_collector".into(),
            StakingModules::BondedPool => staking::BONDED_POOL_NAME.into(),
            StakingModules::NotBondedPool => staking::NOT_BONDED_POOL_NAME.into(),
        }
    }

    fn address(&self) -> AccAddress {
        match self {
            StakingModules::FeeCollector => {
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
            StakingModules::BondedPool => {
                AccAddress::from_bech32("cosmos10q6njatvx5u9jwz2desnzdj6232yx4te2e2h5sekvdj4jwzxx3tkvdntv439wnp32ae5gstzfdkxsdrdwgehyknvx4xhxs6jxqe9g5ttg345wv25fy6rz46gxes56urw232kvs2fwpt5j33k09ekvdntwpg9zu662kyl4s")
                    .expect("hard coded address is valid")
            }
            StakingModules::NotBondedPool => {
                AccAddress::from_bech32("cosmos1w9f8xsn3x48kcatsdpaxg7texqc4zc6vddx5wvt5d3n8vu6gx3c9g668fgc5xsjj29pnvkn5va4k5en6da24zvtk2q69y7tfdq6k7kf4w3p56m6ng3a8w4pexe39wu2fdfz5y4rpv564z6ektpz4gjtdd4fy7snn03rq66")
                    .expect("hard coded address is valid")
            }
        }
    }

    fn permissions(&self) -> Vec<String> {
        match self {
            StakingModules::FeeCollector => Vec::new(),
            StakingModules::BondedPool => vec!["burner".into(), "staking".into()],
            StakingModules::NotBondedPool => {
                vec!["burner".into(), "staking".into()]
            }
        }
    }
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, StoreKeys)]
#[skey(params = Params)]
pub enum SpaceKey {
    #[skey(to_string = "acc")]
    Auth,
    #[skey(to_string = "bank")]
    Bank,
    #[skey(to_string = "params")]
    Params,
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys)]
pub enum SubspaceKey {
    #[pkey(to_string = "auth/")]
    Auth,
    #[pkey(to_string = "bank/")]
    Bank,
    #[pkey(to_string = "baseapp/")]
    BaseApp,
}
