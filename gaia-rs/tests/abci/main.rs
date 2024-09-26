use gaia_rs::abci_handler::GaiaABCIHandler;
use gaia_rs::config::AppConfig;
use gaia_rs::genesis::GenesisState;
use gaia_rs::store_keys::GaiaParamsStoreKey;
use gaia_rs::GaiaApplication;
use gears::baseapp::genesis::Genesis;
use gears::baseapp::options::NodeOptions;
use gears::baseapp::BaseApp;
use gears::config::Config;
use gears::crypto::keys::ReadAccAddress;
use gears::store::database::MemDB;
use gears::tendermint::types::chain_id::ChainId;
use gears::tendermint::types::proto::consensus::ConsensusParams;
use gears::tendermint::types::proto::validator::{ValidatorUpdate, VotingPower};
use gears::tendermint::types::time::timestamp::Timestamp;
use gears::utils::node::{InitState, MockNode, User};
use keyring::key::pair::KeyPair;
use std::fs;
use std::path::Path;

mod scenario_1;
mod scenario_2;
mod scenario_3;
#[cfg(test)]
mod two_tx;

// cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux
const USER_0: &str = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
// cosmos15qzm75pjh0jqsv3u40hzp2vzs2hdp47fkz7j5q
const USER_1: &str = "unfair live spike near cushion blanket club salad poet cigar venue above north speak harbor salute curve tail appear obvious month end boss priority";

// This is a helper function to create a user with a specific account number
pub fn user(account_number: u64, mnemonic: &str) -> User {
    let mnemonic =
        bip32::Mnemonic::new(mnemonic, bip32::Language::English).expect("mnemonic is invalid");
    let key_pair = KeyPair::from_mnemonic(&mnemonic);

    User {
        key_pair,
        account_number,
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
    let mnemonic =
        bip32::Mnemonic::new(mnemonic, bip32::Language::English).expect("mnemonic is invalid");
    let key_pair = KeyPair::from_mnemonic(&mnemonic);
    let address = key_pair.get_address();
    let consensus_key = gears::tendermint::crypto::new_private_key();

    let genesis = if let Some(path) = genesis_path {
        let genesis_state =
            fs::read_to_string(path.as_ref()).expect("failed to read genesis state");
        serde_json::from_str(&genesis_state).expect("invalid genesis")
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
