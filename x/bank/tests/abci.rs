use std::{marker::PhantomData, str::FromStr};

use bank::{BankABCIHandler, GenesisState, Keeper, Message};
use gears::{
    application::handlers::node::ModuleInfo,
    params::{ParamsSubspaceKey, SubspaceParseError},
    store::StoreKey,
    tendermint::types::time::timestamp::Timestamp,
    types::{
        address::AccAddress,
        base::{
            coin::UnsignedCoin,
            coins::{Coins, UnsignedCoins},
        },
        msg::send::MsgSend,
    },
    utils::node::{acc_address, generate_txs, init_node, GenesisSource, MockOptionsFormer},
    x::{
        keepers::auth::{AuthKeeper, AuthParams},
        module::Module,
    },
};

#[test]
/// In this scenario, we test the initialization of the application and execute a few blocks
fn test_init_and_few_blocks() {
    let opt: MockOptionsFormer<
        SubspaceKey,
        BankABCIHandler<
            SpaceKey,
            SubspaceKey,
            MockAuthKeeper<SpaceKey, BankModules>,
            BankModules,
            BankModuleInfo,
        >,
        GenesisState,
    > = MockOptionsFormer::new()
        .abci_handler(BankABCIHandler::new(Keeper::new(
            SpaceKey::Auth,
            SubspaceKey::Auth,
            MockAuthKeeper::<_, BankModules>(PhantomData),
        )))
        .baseapp_sbs_key(SubspaceKey::BaseApp)
        .genesis(GenesisSource::Genesis(GenesisState::default()));

    let (mut node, _) = init_node(opt);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "079ca947e30b69479b21da61e1cb9bad4ff5c8ec99dc3d9e32919179f6604a1d"
    );

    for _ in 0..100 {
        node.step(vec![], Timestamp::UNIX_EPOCH);
    }

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "079ca947e30b69479b21da61e1cb9bad4ff5c8ec99dc3d9e32919179f6604a1d"
    );
}

#[test]
/// In this scenario, we test the initialization of the application and execute a tx
fn test_init_and_sending_tx() {
    let mut genesis = GenesisState::default();

    genesis.add_genesis_account(
        acc_address(),
        UnsignedCoins::new(vec![UnsignedCoin::from_str("30uatom").unwrap()]).unwrap(),
    );

    let opt: MockOptionsFormer<
        SubspaceKey,
        BankABCIHandler<
            SpaceKey,
            SubspaceKey,
            MockAuthKeeper<SpaceKey, BankModules>,
            BankModules,
            BankModuleInfo,
        >,
        GenesisState,
    > = MockOptionsFormer::new()
        .abci_handler(BankABCIHandler::new(Keeper::new(
            SpaceKey::Auth,
            SubspaceKey::Auth,
            MockAuthKeeper::<_, BankModules>(PhantomData),
        )))
        .baseapp_sbs_key(SubspaceKey::BaseApp)
        .genesis(GenesisSource::Genesis(genesis));

    let (mut node, user) = init_node(opt);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "7422bab46c0294d81bcf5fca0495c114a8e40ddd0601539775e5c03f479ad289"
    );

    node.step(vec![], Timestamp::UNIX_EPOCH);
    node.step(vec![], Timestamp::UNIX_EPOCH);

    let to_address = "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut"
        .parse()
        .expect("hard coded address is valid");
    let amount = Coins::new(vec!["10uatom".parse().expect("hard coded coin is valid")])
        .expect("hard coded coins are valid");

    let msg = Message::Send(MsgSend {
        from_address: user.address(),
        to_address,
        amount,
    });

    let txs = generate_txs([(0, msg)], &user, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::UNIX_EPOCH);
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "f9da1d84dcdbd650d3be54bb6fd02ce74c94667922aa9911bd96ca397f4d4e38"
    );
}

#[derive(Debug, Clone)]
struct BankModuleInfo;

impl ModuleInfo for BankModuleInfo {
    const NAME: &'static str = "bank";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BankModules {
    FeeCollector,
}

impl Module for BankModules {
    fn get_name(&self) -> String {
        match self {
            BankModules::FeeCollector => "fee_collector".into(),
        }
    }

    fn get_address(&self) -> AccAddress {
        match self {
            BankModules::FeeCollector => {
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
        }
    }

    fn get_permissions(&self) -> Vec<String> {
        match self {
            BankModules::FeeCollector => vec![],
        }
    }
}

#[derive(Debug, Clone)]
struct MockAuthKeeper<SK, M>(pub PhantomData<(SK, M)>);

struct MockAuthParams;

impl AuthParams for MockAuthParams {
    fn max_memo_characters(&self) -> u64 {
        todo!()
    }

    fn sig_verify_cost_secp256k1(&self) -> u64 {
        todo!()
    }

    fn tx_cost_per_byte(&self) -> u64 {
        todo!()
    }
}

impl<SK: StoreKey, M: Module> AuthKeeper<SK, M> for MockAuthKeeper<SK, M> {
    type Params = MockAuthParams;

    fn get_auth_params<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SK>,
    >(
        &self,
        _: &CTX,
    ) -> Result<Self::Params, gears::types::store::gas::errors::GasStoreErrors> {
        todo!()
    }

    fn has_account<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SK>,
    >(
        &self,
        _: &CTX,
        _: &AccAddress,
    ) -> Result<bool, gears::types::store::gas::errors::GasStoreErrors> {
        Ok(true)
    }

    fn get_account<
        DB: gears::store::database::Database,
        CTX: gears::context::QueryableContext<DB, SK>,
    >(
        &self,
        _: &CTX,
        _: &AccAddress,
    ) -> Result<
        Option<gears::types::account::Account>,
        gears::types::store::gas::errors::GasStoreErrors,
    > {
        todo!()
    }

    fn set_account<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: gears::types::account::Account,
    ) -> Result<(), gears::types::store::gas::errors::GasStoreErrors> {
        todo!()
    }

    fn create_new_base_account<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: &AccAddress,
    ) -> Result<(), gears::types::store::gas::errors::GasStoreErrors> {
        todo!()
    }

    fn check_create_new_module_account<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: &M,
    ) -> Result<(), gears::types::store::gas::errors::GasStoreErrors> {
        todo!()
    }
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum SpaceKey {
    Auth,
    Bank,
    Params,
}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum SubspaceKey {
    Auth,
    Bank,
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
            SubspaceKey::Bank => "bank/",
        }
    }
}

impl StoreKey for SpaceKey {
    fn name(&self) -> &'static str {
        match self {
            SpaceKey::Auth => "acc",
            SpaceKey::Params => "params",
            SpaceKey::Bank => "bank",
        }
    }

    fn params() -> &'static Self {
        const PARAM_KEY: SpaceKey = SpaceKey::Params;

        &PARAM_KEY
    }
}
