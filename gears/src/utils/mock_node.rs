use std::path::PathBuf;

use address::AccAddress;
use bytes::Bytes;

use core_types::Protobuf as _;
use database::MemDB;
use keyring::key::pair::KeyPair;
use prost::Message;
use tendermint::{
    application::ABCIApplication,
    types::{
        chain_id::ChainId,
        proto::{
            block::BlockId,
            consensus::{Consensus, ConsensusParams},
            header::{Header, PartSetHeader},
            info::LastCommitInfo,
            validator::{ValidatorUpdate, VotingPower},
        },
        request::{
            begin_block::RequestBeginBlock, deliver_tx::RequestDeliverTx,
            end_block::RequestEndBlock, init_chain::RequestInitChain,
        },
        time::timestamp::Timestamp,
    },
};

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    baseapp::{genesis::Genesis, options::NodeOptions, BaseApp},
    crypto::{info::SigningInfo, keys::ReadAccAddress},
    params::ParamsSubspaceKey,
    types::{
        auth::fee::Fee,
        base::coins::Coins,
        tx::{body::TxBody, TxMessage},
    },
};

pub struct User {
    pub key_pair: KeyPair,
    pub account_number: u64,
}

impl User {
    pub fn address(&self) -> AccAddress {
        self.key_pair.get_address()
    }
}

#[derive(Debug, Clone)]
pub struct MockApplication;

impl ApplicationInfo for MockApplication {}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub struct InitState<G> {
    pub time: Timestamp,
    pub chain_id: ChainId,
    pub consensus_params: ConsensusParams,
    pub validators: Vec<ValidatorUpdate>,
    pub app_genesis: G,
    pub initial_height: u32,
}

impl<G> From<InitState<G>> for RequestInitChain<G> {
    fn from(init_state: InitState<G>) -> Self {
        Self {
            time: init_state.time,
            chain_id: init_state.chain_id,
            consensus_params: init_state.consensus_params,
            validators: init_state.validators,
            app_genesis: init_state.app_genesis,
            initial_height: init_state.initial_height,
        }
    }
}

pub struct MockNode<App, G> {
    app: App,
    app_hash: Bytes,
    height: u32,
    chain_id: ChainId,
    time: Timestamp,
    last_block_id: BlockId,
    // last_header: Header,
    _phantom: std::marker::PhantomData<G>,
}

impl<G: Clone, App: ABCIApplication<G>> MockNode<App, G> {
    pub fn new(app: App, init_state: InitState<G>) -> Self {
        let res = app.init_chain(init_state.clone().into());

        Self {
            app,
            app_hash: res.app_hash,
            height: 0,
            chain_id: init_state.chain_id,
            time: init_state.time,
            last_block_id: BlockId {
                hash: vec![],
                part_set_header: Some(PartSetHeader {
                    total: 0,
                    hash: vec![],
                }),
            },
            _phantom: Default::default(),
        }
    }

    pub fn step(&mut self, txs: Vec<Bytes>, block_time: Timestamp) -> &Bytes {
        let header = self.calculate_header();
        self.height += 1;
        self.time = block_time;
        // TODO: update last_block_id

        let request_begin_block = RequestBeginBlock {
            header,
            last_commit_info: LastCommitInfo {
                round: 0,
                votes: vec![],
            },
            byzantine_validators: vec![],
            hash:  b"\xaaw\xbd^\x9d\x041\xfdc\x17\x11\x82\xb9iU\xde2\xd0\x19\xca\xdeV\x0e\x7fK\x1c\x88\xb6\xa3\xe3\x8b\x89".as_slice().into(),
        };
        self.app.begin_block(request_begin_block);

        for tx in txs {
            let res = self.app.deliver_tx(RequestDeliverTx { tx });

            if res.code != 0 {
                eprintln!("Error: {:?}", res.log);
            }

            assert!(res.code == 0);
        }

        self.app.end_block(RequestEndBlock {
            height: self.height as i64,
        });

        let res_commit = self.app.commit();

        self.app_hash = res_commit.data;

        &self.app_hash
    }

    fn calculate_header(&self) -> Header {
        Header {
            version: Consensus { block: 11, app: 10 },
            chain_id: self.chain_id.clone(),
            height: self.height,
            time: self.time,
            last_block_id: self.last_block_id.clone(),
            //TODO: need to calculate this
            last_commit_hash: vec![
                227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39,
                174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85,
            ],
            //TODO: need to calculate this
            data_hash: vec![
                227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39,
                174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85,
            ],
            //TODO: need to calculate this
            validators_hash: vec![
                105, 109, 157, 224, 221, 36, 139, 200, 18, 31, 171, 146, 191, 69, 50, 98, 210, 209,
                111, 225, 255, 132, 34, 75, 183, 135, 230, 89, 52, 173, 104, 13,
            ],
            //TODO: need to calculate this
            next_validators_hash: vec![
                105, 109, 157, 224, 221, 36, 139, 200, 18, 31, 171, 146, 191, 69, 50, 98, 210, 209,
                111, 225, 255, 132, 34, 75, 183, 135, 230, 89, 52, 173, 104, 13,
            ],
            //TODO: need to calculate this
            consensus_hash: vec![
                4, 128, 145, 188, 125, 220, 40, 63, 119, 191, 191, 145, 215, 60, 68, 218, 88, 195,
                223, 138, 156, 188, 134, 116, 5, 216, 183, 243, 218, 173, 162, 47,
            ],
            app_hash: self.app_hash.clone().into(),
            //TODO: need to calculate this
            last_results_hash: vec![
                227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39,
                174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85,
            ],
            //TODO: need to calculate this
            evidence_hash: vec![
                227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39,
                174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85,
            ],
            //TODO: need to calculate this
            proposer_address: vec![
                139, 66, 235, 161, 172, 24, 201, 229, 172, 156, 56, 187, 215, 206, 138, 87, 207,
                173, 214, 85,
            ],
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }
}

pub fn generate_txs<M: TxMessage>(
    msgs: impl IntoIterator<Item = (u64, M)>,
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

        let raw_tx = crate::crypto::info::create_signed_transaction_direct(
            vec![signing_info],
            chain_id.to_owned(),
            fee.to_owned(),
            None,
            body_bytes,
        )
        .expect("returns infallible result");

        result.push(
            core_types::tx::raw::TxRaw::from(raw_tx)
                .encode_to_vec()
                .into(),
        )
    }

    result
}

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

pub fn init<PSK: ParamsSubspaceKey, H: ABCIHandler<Genesis = GS>, GS: Genesis>(
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
    let mnemonic = bip32::Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
    let key_pair = KeyPair::from_mnemonic(&mnemonic);
    let address = key_pair.get_address();
    let consensus_key = crate::tendermint::crypto::new_private_key();

    let app_genesis = match genesis {
        GenesisSource::File(path) => {
            println!("Loading genesis state from {:?}", path.as_path());
            // print current directory
            let current_dir = std::env::current_dir().unwrap();
            println!("The current directory is {}", current_dir.display());
            let genesis_state = std::fs::read_to_string(path.as_path()).unwrap();
            serde_json::from_str(&genesis_state).unwrap()
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
