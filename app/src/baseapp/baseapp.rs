use std::sync::{Arc, RwLock};

use database::{RocksDB, DB};
use ibc_proto::cosmos::tx::v1beta1::Tx;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

use bytes::Bytes;
use proto_messages::cosmos::auth::v1beta1::QueryAccountRequest;
use proto_messages::cosmos::bank::v1beta1::QueryAllBalancesRequest;
use proto_messages::cosmos::base::v1beta1::Coin;
use proto_messages::cosmos::{
    auth::v1beta1::BaseAccount,
    base::v1beta1::{Coin as ProtoCoin, SendCoins},
};
use proto_types::AccAddress;
use tendermint_abci::Application;
use tendermint_proto::abci::{
    Event, EventAttribute, RequestApplySnapshotChunk, RequestBeginBlock, RequestCheckTx,
    RequestDeliverTx, RequestEcho, RequestEndBlock, RequestInfo, RequestInitChain,
    RequestLoadSnapshotChunk, RequestOfferSnapshot, RequestQuery, ResponseApplySnapshotChunk,
    ResponseBeginBlock, ResponseCheckTx, ResponseCommit, ResponseDeliverTx, ResponseEcho,
    ResponseEndBlock, ResponseFlush, ResponseInfo, ResponseInitChain, ResponseListSnapshots,
    ResponseLoadSnapshotChunk, ResponseOfferSnapshot, ResponseQuery,
};
use tracing::info;

use crate::{
    baseapp::{ante::AnteHandler, params},
    error::AppError,
    store::MultiStore,
    types::{Context, DecodedTx, Msg, QueryContext},
    x::{
        auth::{Auth, DEFAULT_PARAMS},
        bank::{Balance, Bank, GenesisState, DEFAULT_PARAMS as BANK_DEFAULT_PARAMS},
    },
};

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
//TODO:
// 1. Remove unwraps
// 2. Remove "hash goes here"

#[derive(Debug, Clone)]
pub struct BaseApp {
    multi_store: Arc<RwLock<MultiStore<RocksDB>>>,
    height: Arc<RwLock<u64>>,
}

impl BaseApp {
    pub fn new(db: RocksDB) -> Self {
        let multi_store = MultiStore::new(db);
        let height = multi_store.get_head_version().into();
        Self {
            multi_store: Arc::new(RwLock::new(multi_store)),
            height: Arc::new(RwLock::new(height)),
        }
    }

    fn get_block_height(&self) -> u64 {
        *self.height.read().expect("RwLock will not be poisoned")
    }

    fn get_last_commit_hash(&self) -> [u8; 32] {
        self.multi_store
            .read()
            .expect("RwLock will not be poisoned")
            .get_head_commit_hash()
    }

    fn increment_block_height(&self) -> u64 {
        let mut height = self.height.write().expect("RwLock will not be poisoned");
        *height += 1;
        return *height;
    }

    fn run_tx(&self, raw: Bytes) -> Result<(), AppError> {
        // TODO:
        // 1. Check from address is signer + verify signature

        //###########################
        let tx = Tx::decode(raw.clone()).unwrap();

        let public = tx.auth_info.clone().unwrap().signer_infos[0]
            .clone()
            .public_key
            .unwrap()
            .type_url;
        println!("################# URL:  {}", public);
        ///cosmos.crypto.secp256k1.PubKey
        // let msgs = tx.get_msgs();
        // let msg = &msgs[0];

        // let signers = msg.get_signers();

        // println!("################### Signers: {}", signers);

        // Ok(())

        //#######################
        let tx = DecodedTx::from_bytes(raw)?;

        Self::validate_basic_tx_msgs(tx.get_msgs())?;

        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");
        let mut ctx = Context::new(&mut multi_store, self.get_block_height());

        match AnteHandler::run(&mut ctx, &tx) {
            Ok(_) => multi_store.write_then_clear_tx_caches(),
            Err(e) => {
                multi_store.clear_tx_caches();
                return Err(e);
            }
        };

        let mut ctx = Context::new(&mut multi_store, self.get_block_height());

        match BaseApp::run_msgs(&mut ctx, tx.get_msgs()) {
            Ok(_) => {
                multi_store.write_then_clear_tx_caches();
                Ok(())
            }
            Err(e) => {
                multi_store.clear_tx_caches();
                Err(e)
            }
        }
    }

    fn run_msgs<T: DB>(ctx: &mut Context<T>, msgs: &Vec<Msg>) -> Result<(), AppError> {
        for msg in msgs {
            match msg {
                Msg::Send(send_msg) => {
                    Bank::send_coins_from_account_to_account(ctx, send_msg.clone())?
                }
                Msg::Test => return Err(AppError::AccountNotFound),
            };
        }

        return Ok(());
    }

    fn validate_basic_tx_msgs(msgs: &Vec<Msg>) -> Result<(), AppError> {
        if msgs.is_empty() {
            return Err(AppError::InvalidRequest(
                "must contain at least one message".into(),
            ));
        }

        for msg in msgs {
            msg.validate_basic()?
        }

        return Ok(());
    }
}

impl Application for BaseApp {
    fn init_chain(&self, request: RequestInitChain) -> ResponseInitChain {
        info!("Got init chain request");
        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");
        let mut ctx = Context::new(&mut multi_store, self.get_block_height());

        if let Some(params) = request.consensus_params.clone() {
            params::set_consensus_params(&mut ctx, params);
        }

        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32(
                    &"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
                )
                .expect("hard coded address is valid"),
                coins: vec![Coin {
                    denom: "uatom".to_string().try_into().unwrap(),
                    amount: cosmwasm_std::Uint256::from(34_u32),
                }],
            }],
            total_supply: SendCoins::new(vec![ProtoCoin {
                denom: "uatom".to_string().try_into().unwrap(),
                amount: cosmwasm_std::Uint256::from(34_u32),
            }])
            .unwrap(),
            params: BANK_DEFAULT_PARAMS,
        };
        Bank::init_genesis(&mut ctx, genesis);

        let genesis = crate::x::auth::GenesisState {
            accounts: vec![BaseAccount {
                address: AccAddress::from_bech32(
                    "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into(),
                )
                .expect("hard coded address is valid"),
                pub_key: None,
                account_number: 0,
                sequence: 0,
            }],
            params: DEFAULT_PARAMS,
        };
        Auth::init_genesis(&mut ctx, genesis).expect("genesis is valid");

        multi_store.write_then_clear_tx_caches();

        ResponseInitChain {
            consensus_params: request.consensus_params,
            validators: request.validators,
            app_hash: "hash_goes_here".into(),
        }
    }

    fn info(&self, request: RequestInfo) -> ResponseInfo {
        info!(
            "Got info request. Tendermint version: {}; Block version: {}; P2P version: {}",
            request.version, request.block_version, request.p2p_version
        );

        ResponseInfo {
            data: APP_NAME.to_string(),
            version: APP_VERSION.to_string(),
            app_version: 1,
            last_block_height: self
                .get_block_height()
                .try_into()
                .expect("can't believe we made it this far"),
            last_block_app_hash: self.get_last_commit_hash().to_vec().into(),
        }
    }

    fn query(&self, request: RequestQuery) -> ResponseQuery {
        info!("Got query request to: {}", request.path);

        if request.path.starts_with("/ibc") {
            //handle ibc queries
            ResponseQuery {
                code: 1,
                log: "unrecognized query".to_string(),
                info: "".to_string(),
                index: 0,
                key: request.data,
                value: Default::default(),
                proof_ops: None,
                height: 0,
                codespace: "".to_string(),
            }
        } else {
            match request.path.as_str() {
                "/cosmos.bank.v1beta1.Query/AllBalances" => {
                    let data = request.data.clone();
                    let req = QueryAllBalancesRequest::decode(data).unwrap();

                    let store = self.multi_store.read().unwrap();
                    let ctx = QueryContext::new(&store, self.get_block_height());

                    let res = Bank::query_all_balances(&ctx, req);

                    match res {
                        Ok(res) => {
                            let res = res.encode_vec().expect(
                                "library call will never return an error - this is a bug in the library",
                            );

                            ResponseQuery {
                                code: 0,
                                log: "exists".to_string(),
                                info: "".to_string(),
                                index: 0,
                                key: request.data,
                                value: res.into(),
                                proof_ops: None,
                                height: self
                                    .get_block_height()
                                    .try_into()
                                    .expect("can't believe we made it this far"),
                                codespace: "".to_string(),
                            }
                        }
                        Err(e) => ResponseQuery {
                            code: 0,
                            log: e.to_string(),
                            info: "".to_string(),
                            index: 0,
                            key: request.data,
                            value: vec![].into(),
                            proof_ops: None,
                            height: self
                                .get_block_height()
                                .try_into()
                                .expect("can't believe we made it this far"),
                            codespace: "".to_string(),
                        },
                    }
                }
                "/cosmos.auth.v1beta1.Query/Account" => {
                    let data = request.data.clone();
                    let req = QueryAccountRequest::decode(data).unwrap();

                    let store = self.multi_store.read().unwrap();
                    let ctx = QueryContext::new(&store, self.get_block_height());

                    let res = Auth::query_account(&ctx, req);

                    match res {
                        Ok(res) => ResponseQuery {
                            code: 0,
                            log: "exists".to_string(),
                            info: "".to_string(),
                            index: 0,
                            key: request.data,
                            value: res.encode_to_vec().into(),
                            proof_ops: None,
                            height: 0,
                            codespace: "".to_string(),
                        },
                        Err(e) => ResponseQuery {
                            code: 1,
                            log: e.to_string(),
                            info: "".to_string(),
                            index: 0,
                            key: request.data,
                            value: vec![].into(),
                            proof_ops: None,
                            height: 0,
                            codespace: "".to_string(),
                        },
                    }
                }

                _ => ResponseQuery {
                    code: 1,
                    log: "unrecognized query".to_string(),
                    info: "".to_string(),
                    index: 0,
                    key: request.data,
                    value: Default::default(),
                    proof_ops: None,
                    height: 0,
                    codespace: "".to_string(),
                },
            }
        }
    }

    fn check_tx(&self, _request: RequestCheckTx) -> ResponseCheckTx {
        info!("Got check tx request");
        ResponseCheckTx {
            code: 0,
            data: Default::default(),
            log: "".to_string(),
            info: "".to_string(),
            gas_wanted: 1,
            gas_used: 0,
            events: vec![],
            codespace: "".to_string(),
            mempool_error: "".to_string(),
            priority: 0,
            sender: "".to_string(),
        }
    }

    fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx {
        info!("Got deliver tx request");
        match self.run_tx(request.tx) {
            Ok(_) => ResponseDeliverTx {
                code: 0,
                data: Default::default(),
                log: "".to_string(),
                info: "".to_string(),
                gas_wanted: 0,
                gas_used: 0,
                events: vec![Event {
                    r#type: "app".to_string(),
                    attributes: vec![EventAttribute {
                        key: "key".into(),
                        value: "nothing".into(),
                        index: true,
                    }],
                }],
                codespace: "".to_string(),
            },
            Err(e) => {
                info!("Failed to process tx: {}", e);
                ResponseDeliverTx {
                    code: e.code(),
                    data: Bytes::new(),
                    log: e.to_string(),
                    info: "".to_string(),
                    gas_wanted: 0,
                    gas_used: 0,
                    events: vec![],
                    codespace: "".to_string(),
                }
            }
        }
    }

    fn commit(&self) -> ResponseCommit {
        info!("Got commit request");
        let new_height = self.increment_block_height();
        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");

        let hash = multi_store.commit();
        info!(
            "Committed state, block height: {} app hash: {}",
            new_height,
            hex::encode(hash)
        );

        ResponseCommit {
            data: hash.to_vec().into(),
            retain_height: (new_height - 1)
                .try_into()
                .expect("can't believe we made it this far"),
        }
    }

    fn echo(&self, request: RequestEcho) -> ResponseEcho {
        info!("Got echo request");
        ResponseEcho {
            message: request.message,
        }
    }

    fn begin_block(&self, _request: RequestBeginBlock) -> ResponseBeginBlock {
        info!("Got begin block request");
        Default::default()
    }

    fn end_block(&self, _request: RequestEndBlock) -> ResponseEndBlock {
        info!("Got end block request");
        Default::default()
    }

    /// Signals that messages queued on the client should be flushed to the server.
    fn flush(&self) -> ResponseFlush {
        info!("Got flush request");
        ResponseFlush {}
    }

    /// Used during state sync to discover available snapshots on peers.
    fn list_snapshots(&self) -> ResponseListSnapshots {
        info!("Got list snapshots request");
        Default::default()
    }

    /// Called when bootstrapping the node using state sync.
    fn offer_snapshot(&self, _request: RequestOfferSnapshot) -> ResponseOfferSnapshot {
        info!("Got offer snapshot request");
        Default::default()
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    fn load_snapshot_chunk(&self, _request: RequestLoadSnapshotChunk) -> ResponseLoadSnapshotChunk {
        info!("Got load snapshot chunk request");
        Default::default()
    }

    /// Apply the given snapshot chunk to the application's state.
    fn apply_snapshot_chunk(
        &self,
        _request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        info!("Got apply snapshot chunk request");
        Default::default()
    }
}
