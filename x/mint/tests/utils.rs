use gears::{
    application::handlers::node::ModuleInfo,
    baseapp::BaseApp,
    derive::{ParamsKeys, StoreKeys},
    store::database::MemDB,
    utils::node::{GenesisSource, MockApplication, MockNode, MockOptions},
    x::{
        keepers::{
            bank::{BalancesKeeper, BankKeeper},
            mint::{MintingBankKeeper, MintingStakingKeeper},
        },
        module::Module,
    },
};
use mint::{abci_handler::MintAbciHandler, genesis::MintGenesis, keeper::MintKeeper};

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, StoreKeys)]
#[skey(params = Params)]
pub enum SpaceKey {
    #[skey(to_string = "acc")]
    Auth,
    #[skey(to_string = "bank")]
    Bank,
    #[skey(to_string = "mint")]
    Mint,
    #[skey(to_string = "params")]
    Params,
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys)]
pub enum SubspaceKey {
    #[pkey(to_string = "auth/")]
    Auth,
    #[pkey(to_string = "bank/")]
    Bank,
    #[pkey(to_string = "mint/")]
    Mint,
    #[pkey(to_string = "baseapp/")]
    BaseApp,
}

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumIter)]
pub enum Modules {
    FeeCollector,
    Mint,
}

impl Module for Modules {
    fn name(&self) -> String {
        match self {
            Modules::FeeCollector => "fee_collector".into(),
            Modules::Mint => "mint".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MintModuleInfo;

impl ModuleInfo for MintModuleInfo {
    const NAME: &'static str = "mint";
}

pub fn set_node() -> MockNode<
    BaseApp<
        MemDB,
        SubspaceKey,
        MintAbciHandler<
            SpaceKey,
            SubspaceKey,
            MockBankKeeper,
            MockStakingKeeper,
            Modules,
            MintModuleInfo,
        >,
        MockApplication,
    >,
    MintGenesis,
> {
    let handler = MintAbciHandler::new(
        MintKeeper::new(
            SpaceKey::Mint,
            MockStakingKeeper,
            MockBankKeeper,
            Modules::Mint,
            Modules::FeeCollector,
        ),
        SubspaceKey::Mint,
    );

    let _opt = MockOptions::<
        SubspaceKey,
        MintAbciHandler<_, _, _, _, _, MintModuleInfo>,
        MintGenesis,
    >::former()
    .abci_handler(handler)
    .baseapp_sbs_key(SubspaceKey::BaseApp)
    .genesis(GenesisSource::Default);

    // init_node(opt)
    todo!()
}

#[derive(Debug, Clone)]
pub struct MockStakingKeeper;

impl MintingStakingKeeper<SpaceKey, Modules> for MockStakingKeeper {
    fn staking_denom<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &CTX,
    ) -> Result<gears::types::denom::Denom, gears::types::store::gas::errors::GasStoreErrors> {
        todo!()
    }

    fn total_bonded_tokens<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &CTX,
    ) -> Result<
        gears::types::decimal256::Decimal256,
        gears::types::store::gas::errors::GasStoreErrors,
    > {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct MockBankKeeper;

impl MintingBankKeeper<SpaceKey, Modules> for MockBankKeeper {
    fn mint_coins<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &mut CTX,
        _module: &Modules,
        _amount: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::x::errors::BankKeeperError> {
        todo!()
    }
}

impl BankKeeper<SpaceKey, Modules> for MockBankKeeper {
    fn send_coins_from_account_to_module<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &mut CTX,
        _from_address: gears::types::address::AccAddress,
        _to_module: &Modules,
        _amount: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::x::errors::BankKeeperError> {
        todo!()
    }

    fn send_coins_from_module_to_account<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &mut CTX,
        _address: &gears::types::address::AccAddress,
        _module: &Modules,
        _amount: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::x::errors::BankKeeperError> {
        todo!()
    }

    fn send_coins_from_module_to_module<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &mut CTX,
        _sender_pool: &Modules,
        _recepient_pool: &Modules,
        _amount: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::x::errors::BankKeeperError> {
        todo!()
    }

    fn denom_metadata<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &CTX,
        _base: &gears::types::denom::Denom,
    ) -> Result<
        Option<gears::types::tx::metadata::Metadata>,
        gears::types::store::gas::errors::GasStoreErrors,
    > {
        todo!()
    }

    fn coins_burn<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &mut CTX,
        _module: &Modules,
        _deposit: &gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::x::errors::BankKeeperError> {
        todo!()
    }
}

impl BalancesKeeper<SpaceKey, Modules> for MockBankKeeper {
    fn balance_all<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &CTX,
        _address: gears::types::address::AccAddress,
        _pagination: Option<gears::extensions::pagination::Pagination>,
    ) -> Result<
        (
            Option<gears::extensions::pagination::PaginationResult>,
            Vec<gears::types::base::coin::UnsignedCoin>,
        ),
        gears::types::store::gas::errors::GasStoreErrors,
    > {
        todo!()
    }

    fn supply<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SpaceKey>,
    >(
        &self,
        _ctx: &CTX,
        _denom: &gears::types::denom::Denom,
    ) -> Result<
        Option<gears::types::base::coin::UnsignedCoin>,
        gears::types::store::gas::errors::GasStoreErrors,
    > {
        todo!()
    }
}
