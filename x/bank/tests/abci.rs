use std::str::FromStr;

use bank::{BankABCIHandler, GenesisState, Keeper, Message};
use gears::{
    application::handlers::node::ModuleInfo,
    derive::{ParamsKeys, StoreKeys},
    extensions::testing::UnwrapTesting,
    tendermint::types::time::timestamp::Timestamp,
    types::{
        address::AccAddress,
        base::{
            coin::UnsignedCoin,
            coins::{Coins, UnsignedCoins},
        },
        msg::send::MsgSend,
    },
    utils::node::{acc_address, generate_tx, init_node, GenesisSource, MockOptionsFormer, User},
    x::{keepers::mocks::auth::MockAuthKeeper, module::Module},
};

#[test]
/// In this scenario, we test the initialization of the application and execute a few blocks
fn test_init_and_few_blocks() {
    let opt: MockOptionsFormer<
        SubspaceKey,
        BankABCIHandler<SpaceKey, SubspaceKey, MockAuthKeeper, BankModules, BankModuleInfo>,
        GenesisState,
    > = MockOptionsFormer::new()
        .abci_handler(BankABCIHandler::new(Keeper::new(
            SpaceKey::Auth,
            SubspaceKey::Auth,
            MockAuthKeeper::former().form(),
        )))
        .baseapp_sbs_key(SubspaceKey::BaseApp)
        .genesis(GenesisSource::Genesis(GenesisState::default()));

    let mut node = init_node(opt);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "079ca947e30b69479b21da61e1cb9bad4ff5c8ec99dc3d9e32919179f6604a1d"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
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
        UnsignedCoins::new(vec![UnsignedCoin::from_str("30uatom").unwrap_test()]).unwrap_test(),
    );

    let opt: MockOptionsFormer<
        SubspaceKey,
        BankABCIHandler<SpaceKey, SubspaceKey, MockAuthKeeper, BankModules, BankModuleInfo>,
        GenesisState,
    > = MockOptionsFormer::new()
        .abci_handler(BankABCIHandler::new(Keeper::new(
            SpaceKey::Auth,
            SubspaceKey::Auth,
            MockAuthKeeper::former().form(),
        )))
        .baseapp_sbs_key(SubspaceKey::BaseApp)
        .genesis(GenesisSource::Genesis(genesis));

    let mut node = init_node(opt);

    let user = User::from_bech32("race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow", 1).unwrap_test();

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
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

    let txs = generate_tx(vec1::vec1![msg], 0, &user, node.chain_id().clone());

    let app_hash = &node.step(vec![txs], Timestamp::UNIX_EPOCH).app_hash;
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

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumIter)]
pub enum BankModules {
    FeeCollector,
}

impl Module for BankModules {
    fn name(&self) -> String {
        match self {
            BankModules::FeeCollector => "fee_collector".into(),
        }
    }

    fn address(&self) -> AccAddress {
        match self {
            BankModules::FeeCollector => {
                AccAddress::from_bech32("cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta")
                    .expect("hard coded address is valid")
            }
        }
    }

    fn permissions(&self) -> Vec<String> {
        match self {
            BankModules::FeeCollector => Vec::new(),
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
