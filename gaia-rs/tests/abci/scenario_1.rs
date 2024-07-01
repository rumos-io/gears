use gaia_rs::abci_handler::GaiaABCIHandler;

use gaia_rs::config::AppConfig;
use gaia_rs::genesis::GenesisState;
use gaia_rs::store_keys::GaiaParamsStoreKey;
use gaia_rs::GaiaApplication;
use gears::baseapp::genesis::Genesis;
use gears::baseapp::options::NodeOptions;
use gears::baseapp::BaseApp;
use gears::config::Config;
use gears::core::tx::raw::TxRaw;
use gears::crypto::info::SigningInfo;
use gears::crypto::keys::ReadAccAddress;
use gears::store::database::MemDB;
use gears::tendermint::mock::{InitState, MockTendermint};
use gears::tendermint::types::chain_id::ChainId;
use gears::tendermint::types::proto::consensus::ConsensusParams;
use gears::tendermint::types::proto::validator::ValidatorUpdate;
use gears::tendermint::types::proto::Protobuf;
use gears::tendermint::types::time::Timestamp;
use gears::types::auth::fee::Fee;
use gears::types::base::send::SendCoins;
use gears::types::msg::send::MsgSend;
use gears::types::tx::body::TxBody;
use keyring::key::pair::KeyPair;
use prost::Message;

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three.
fn scenario_1() {
    let db = MemDB::new();
    let node_options = NodeOptions::default();
    let config: Config<AppConfig> = Config::default();
    let app: BaseApp<MemDB, GaiaParamsStoreKey, GaiaABCIHandler, GaiaApplication> = BaseApp::new(
        db,
        GaiaParamsStoreKey::BaseApp,
        GaiaABCIHandler::new(config),
        node_options,
    );
    let chain_id = ChainId::default();

    let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
    let mnemonic = bip32::Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
    let key_pair = KeyPair::from_mnemonic(&mnemonic);
    let address = key_pair.get_address();
    let consensus_key = gears::tendermint::crypto::new_private_key();

    let mut genesis = GenesisState::default();
    genesis
        .add_genesis_account(
            address.clone(),
            "34uatom".parse().expect("hard coded coin is valid"),
        )
        .expect("won't fail since there's no existing account");

    let init_state = InitState {
        time: Timestamp::ZERO,
        chain_id: chain_id.clone(),
        consensus_params: ConsensusParams::default(),
        validators: vec![ValidatorUpdate {
            pub_key: consensus_key
                .try_into()
                .expect("ed25519 key conversion is supported"),
            power: 10,
        }],
        app_genesis: genesis,
        initial_height: 1,
    };

    let mut node = MockTendermint::new(app, init_state);

    let app_hash = node.step(
        vec![],
        Timestamp {
            seconds: 0,
            nanos: 0,
        },
    );
    assert_eq!(
        hex::encode(app_hash),
        "d0254da38fc9c97292f65f4e8af3276209c6d6f8a922bbad8fc4a8f36af55f67"
    );

    node.step(vec![], Timestamp::ZERO);
    node.step(vec![], Timestamp::ZERO);

    let to_address = "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut"
        .parse()
        .expect("hard coded address is valid");
    let amount = SendCoins::new(vec!["10uatom".parse().expect("hard coded coin is valid")])
        .expect("hard coded coins are valid");

    let msg = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: address,
        to_address,
        amount,
    }));

    let fee = Fee {
        amount: Some(
            SendCoins::new(vec!["1uatom".parse().expect("hard coded coin is valid")])
                .expect("hard coded coins are valid"),
        ),
        gas_limit: 200_000_u64
            .try_into()
            .expect("hard coded gas limit is valid"),
        payer: None,
        granter: "".into(),
    };

    let signing_info = SigningInfo {
        key: key_pair,
        sequence: 0,
        account_number: 2,
    };

    let body_bytes = TxBody::new_with_defaults(vec![msg])
        .encode_vec()
        .expect("can't fail");

    let tip = None;

    let raw_tx = gears::crypto::info::create_signed_transaction_direct(
        vec![signing_info],
        chain_id,
        fee,
        tip,
        body_bytes,
    )
    .expect("returns infallible result");

    let txs = vec![TxRaw::from(raw_tx).encode_to_vec().into()];

    let app_hash = node.step(txs, Timestamp::ZERO);
    assert_eq!(
        hex::encode(app_hash),
        "7bc0e95da6ba637bddaade5e6911fdb20030956a4bb8e305fb1c390ff7bcea20"
    );
}
