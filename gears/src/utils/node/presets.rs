use std::path::PathBuf;

use database::MemDB;
use keyring::key::pair::KeyPair;
use tendermint::types::{
    chain_id::ChainId,
    proto::{
        consensus::ConsensusParams,
        validator::{ValidatorUpdate, VotingPower},
    },
    time::timestamp::Timestamp,
};

use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{genesis::Genesis, options::NodeOptions, BaseApp},
    crypto::keys::ReadAccAddress,
    params::ParamsSubspaceKey,
};

use super::{InitState, MockApplication, MockNode, User};

#[derive(Debug, Clone, Default)]
pub enum GenesisSource<GS> {
    File(PathBuf),
    Genesis(GS),
    #[default]
    Default,
}

#[derive(Debug, Clone, former::Former)]
pub struct MockOptions<PSK: ParamsSubspaceKey, H: ABCIHandler, GS: Genesis> {
    pub baseapp_sbs_key: PSK,
    pub node_opt: Option<NodeOptions>,
    pub genesis: GenesisSource<GS>,
    pub abci_handler: H,
}

impl<PSK: ParamsSubspaceKey, H: ABCIHandler, GS: Genesis> From<MockOptionsFormer<PSK, H, GS>>
    for MockOptions<PSK, H, GS>
{
    fn from(value: MockOptionsFormer<PSK, H, GS>) -> Self {
        value.form()
    }
}

pub fn init_node<PSK: ParamsSubspaceKey, H: ABCIHandler<Genesis = GS>, GS: Genesis>(
    opt: impl Into<MockOptions<PSK, H, GS>>,
) -> (MockNode<BaseApp<MemDB, PSK, H, MockApplication>, GS>, User) {
    let MockOptions {
        baseapp_sbs_key,
        node_opt,
        genesis,
        abci_handler,
    } = opt.into();

    let db = MemDB::new();
    let node_options = node_opt.unwrap_or_default();

    let app: BaseApp<MemDB, PSK, H, MockApplication> =
        BaseApp::new(db, baseapp_sbs_key, abci_handler, node_options);
    let chain_id = ChainId::default();

    let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
    let mnemonic =
        bip32::Mnemonic::new(mnemonic, bip32::Language::English).expect("Invalid mnemonic");
    let key_pair = KeyPair::from_mnemonic(&mnemonic);
    let address = key_pair.get_address();
    let consensus_key = tendermint::crypto::new_private_key();

    let app_genesis = match genesis {
        GenesisSource::File(path) => {
            println!("Loading genesis state from {:?}", path.as_path());
            // print current directory
            let current_dir = std::env::current_dir().expect("failed to get current dir");
            println!("The current directory is {}", current_dir.display());
            let genesis_state =
                std::fs::read_to_string(path.as_path()).expect("failed to read genesis");
            serde_json::from_str(&genesis_state).expect("failed to deserialize genesis state")
        }
        GenesisSource::Genesis(genesis) => genesis,
        GenesisSource::Default => {
            let mut genesis = GS::default();
            genesis
                .add_genesis_account(
                    address.clone(),
                    "34uatom".parse().expect("hard coded coin is valid"),
                )
                .expect("won't fail since there's no existing account");
            genesis
        }
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
        app_genesis,
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
