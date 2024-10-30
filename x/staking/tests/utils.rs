use gears::{
    application::handlers::node::ModuleInfo,
    baseapp::BaseApp,
    derive::{ParamsKeys, StoreKeys},
    store::database::MemDB,
    utils::node::{init_node, GenesisSource, MockApplication, MockNode, MockOptionsFormer},
    x::{
        keepers::mocks::{auth::MockAuthKeeper, bank::MockBankKeeper},
        module::Module,
    },
};
use staking::{GenesisState, Keeper, MockHookKeeper, StakingABCIHandler};

#[derive(Debug, Clone)]
pub struct StakingModuleInfo;

impl ModuleInfo for StakingModuleInfo {
    const NAME: &'static str = "bank";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakingModules {
    BondedPool,
    NotBondedPool,
}

impl Module for StakingModules {
    fn name(&self) -> String {
        match self {
            StakingModules::BondedPool => staking::BONDED_POOL_NAME.into(),
            StakingModules::NotBondedPool => staking::NOT_BONDED_POOL_NAME.into(),
        }
    }

    fn permissions(&self) -> Vec<String> {
        match self {
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
    #[skey(to_string = "staking")]
    Staking,
    #[skey(to_string = "params")]
    Params,
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys)]
pub enum SubspaceKey {
    #[pkey(to_string = "auth/")]
    Auth,
    #[pkey(to_string = "bank/")]
    Bank,
    #[pkey(to_string = "staking/")]
    Staking,
    #[pkey(to_string = "baseapp/")]
    BaseApp,
}

pub fn set_node() -> MockNode<
    BaseApp<
        MemDB,
        SubspaceKey,
        StakingABCIHandler<
            SpaceKey,
            SubspaceKey,
            MockAuthKeeper,
            MockBankKeeper,
            MockHookKeeper<SpaceKey, MockAuthKeeper, StakingModules>,
            StakingModules,
            StakingModuleInfo,
        >,
        MockApplication,
    >,
    GenesisState,
> {
    let handler = StakingABCIHandler::new(Keeper::new(
        SpaceKey::Staking,
        SubspaceKey::Staking,
        MockAuthKeeper::former().form(),
        MockBankKeeper::former().form(),
        Option::<MockHookKeeper<SpaceKey, MockAuthKeeper, StakingModules>>::None,
        StakingModules::BondedPool,
        StakingModules::NotBondedPool,
    ));

    let opt: MockOptionsFormer<
        SubspaceKey,
        StakingABCIHandler<_, SubspaceKey, _, _, _, _, StakingModuleInfo>,
        GenesisState,
    > = MockOptionsFormer::new()
        .abci_handler(handler)
        .baseapp_sbs_key(SubspaceKey::BaseApp)
        .genesis(GenesisSource::Default);

    init_node(opt).0
}
