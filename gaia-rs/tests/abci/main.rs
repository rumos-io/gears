use bytes::Bytes;
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
use gears::core::Protobuf;
use gears::crypto::info::SigningInfo;
use gears::crypto::keys::ReadAccAddress;
use gears::store::database::MemDB;
use gears::tendermint::mock::{InitState, MockNode};
use gears::tendermint::types::chain_id::ChainId;
use gears::tendermint::types::proto::consensus::ConsensusParams;
use gears::tendermint::types::proto::validator::{ValidatorUpdate, VotingPower};
use gears::tendermint::types::time::timestamp::Timestamp;
use gears::types::address::AccAddress;
use gears::types::auth::fee::Fee;
use gears::types::base::coins::Coins;
use gears::types::tx::body::TxBody;
use keyring::key::pair::KeyPair;
use prost::Message;
use std::fs;
use std::path::Path;

mod scenario_1;
mod scenario_2;
#[cfg(test)]
mod two_tx;

struct User {
    key_pair: KeyPair,
    account_number: u64,
}

impl User {
    pub fn address(&self) -> AccAddress {
        self.key_pair.get_address()
    }
}

fn setup_mock_node(
    genesis_path: Option<impl AsRef<Path>>,
) -> (
    MockNode<BaseApp<MemDB, GaiaParamsStoreKey, GaiaABCIHandler, GaiaApplication>, GenesisState>,
    User,
) {
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

    let genesis = if let Some(path) = genesis_path {
        println!("Loading genesis state from {:?}", path.as_ref());
        // print current directory
        let current_dir = std::env::current_dir().unwrap();
        println!("The current directory is {}", current_dir.display());
        let genesis_state = fs::read_to_string(path.as_ref()).unwrap();
        serde_json::from_str(&genesis_state).unwrap()
    } else {
        let mut genesis = GenesisState::default();
        genesis
            .add_genesis_account(
                address.clone(),
                "34uatom".parse().expect("hard coded coin is valid"),
            )
            .expect("won't fail since there's no existing account");
        genesis
    };

    let init_state = InitState {
        time: Timestamp::UNIX_EPOCH,
        chain_id: chain_id.clone(),
        consensus_params: ConsensusParams::default(),
        validators: vec![ValidatorUpdate {
            pub_key: consensus_key
                .try_into()
                .expect("ed25519 key conversion is supported"),
            power: VotingPower::new(10).expect("hardcoded power is less the max voting power"),
        }],
        app_genesis: genesis,
        initial_height: 1,
    };

    (
        MockNode::new(app, init_state),
        User {
            key_pair,
            account_number: 2,
        },
    )
}

fn generate_txs(
    msgs: impl IntoIterator<Item = (u64, gaia_rs::message::Message)>,
    user: &User,
    chain_id: ChainId,
) -> Vec<Bytes> {
    let fee = Fee {
        amount: Some(
            Coins::new(vec!["1uatom".parse().expect("hard coded coin is valid")])
                .expect("hard coded coins are valid"),
        ),
        gas_limit: 200_000_u64
            .try_into()
            .expect("hard coded gas limit is valid"),
        payer: None,
        granter: "".into(),
    };

    let mut result = Vec::new();

    for (sequence, msg) in msgs {
        let signing_info = SigningInfo {
            key: &user.key_pair,
            sequence,
            account_number: user.account_number,
        };

        let body_bytes = TxBody::new_with_defaults(vec1::vec1![msg]).encode_vec();

        let raw_tx = gears::crypto::info::create_signed_transaction_direct(
            vec![signing_info],
            chain_id.to_owned(),
            fee.to_owned(),
            None,
            body_bytes,
        )
        .expect("returns infallible result");

        result.push(TxRaw::from(raw_tx).encode_to_vec().into())
    }

    result
}
