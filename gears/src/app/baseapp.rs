use std::str::FromStr;
use std::sync::{Arc, RwLock};

use database::{RocksDB, DB};
use ibc_proto::cosmos::tx::v1beta1::Tx;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

use bytes::Bytes;
use proto_messages::cosmos::auth::v1beta1::QueryAccountRequest;
use proto_messages::cosmos::bank::v1beta1::QueryAllBalancesRequest;
use proto_messages::cosmos::tx::v1beta1::Msg;
use tendermint_abci::Application;
use tendermint_informal::block::Header;
use tendermint_proto::abci::{
    RequestApplySnapshotChunk, RequestBeginBlock, RequestCheckTx, RequestDeliverTx, RequestEcho,
    RequestEndBlock, RequestInfo, RequestInitChain, RequestLoadSnapshotChunk, RequestOfferSnapshot,
    RequestQuery, ResponseApplySnapshotChunk, ResponseBeginBlock, ResponseCheckTx, ResponseCommit,
    ResponseDeliverTx, ResponseEcho, ResponseEndBlock, ResponseFlush, ResponseInfo,
    ResponseInitChain, ResponseListSnapshots, ResponseLoadSnapshotChunk, ResponseOfferSnapshot,
    ResponseQuery,
};
use tracing::{error, info};

use crate::types::{GenesisState, InitContext, TxContext};
use crate::{
    app::{ante::AnteHandler, params},
    error::AppError,
    store::MultiStore,
    types::{Context, DecodedTx, QueryContext},
    x::{auth::Auth, bank::Bank},
};

pub const APP_NAME: &str = env!("CARGO_PKG_NAME"); // TODO: should this be moved to utils?
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
//TODO:
// 1. Remove unwraps
// 2. Remove "hash goes here"

#[derive(Debug, Clone)]
pub struct BaseApp {
    pub multi_store: Arc<RwLock<MultiStore<RocksDB>>>,
    height: Arc<RwLock<u64>>,
    block_header: Arc<RwLock<Option<Header>>>, // passed by Tendermint in call to begin_block
}

impl BaseApp {
    pub fn new(db: RocksDB) -> Self {
        let multi_store = MultiStore::new(db);
        let height = multi_store.get_head_version().into();
        Self {
            multi_store: Arc::new(RwLock::new(multi_store)),
            height: Arc::new(RwLock::new(height)),
            block_header: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_block_height(&self) -> u64 {
        *self.height.read().expect("RwLock will not be poisoned")
    }

    fn get_block_header(&self) -> Option<Header> {
        self.block_header
            .read()
            .expect("RwLock will not be poisoned")
            .clone()
    }

    fn set_block_header(&self, header: Header) {
        let mut current_header = self
            .block_header
            .write()
            .expect("RwLock will not be poisoned");
        *current_header = Some(header);
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

    fn run_tx(&self, raw: Bytes) -> Result<Vec<tendermint_informal::abci::Event>, AppError> {
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
        //cosmos.crypto.secp256k1.PubKey
        // let msgs = tx.get_msgs();
        // let msg = &msgs[0];

        // let signers = msg.get_signers();

        // println!("################### Signers: {}", signers);

        // Ok(())

        //#######################
        let tx = DecodedTx::from_bytes(raw.clone())?;

        Self::validate_basic_tx_msgs(tx.get_msgs())?;

        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");
        let mut ctx = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block"),
            raw.clone().into(),
        );

        match AnteHandler::run(&mut ctx.as_any(), &tx) {
            Ok(_) => multi_store.write_then_clear_tx_caches(),
            Err(e) => {
                multi_store.clear_tx_caches();
                return Err(e);
            }
        };

        let mut ctx = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block"),
            raw.into(),
        );

        match BaseApp::run_msgs(&mut ctx.as_any(), tx.get_msgs()) {
            Ok(_) => {
                let events = ctx.events;
                multi_store.write_then_clear_tx_caches();
                Ok(events)
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
            msg.validate_basic()
                .map_err(|e| AppError::TxValidation(e.to_string()))?
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

        //TODO: handle request height > 1 as is done in SDK

        let mut ctx = InitContext::new(&mut multi_store, self.get_block_height(), request.chain_id);

        if let Some(params) = request.consensus_params.clone() {
            params::set_consensus_params(&mut ctx.as_any(), params);
        }

        let genesis = String::from_utf8(request.app_state_bytes.into())
            .map_err(|e| AppError::Genesis(e.to_string()))
            .and_then(|f| GenesisState::from_str(&f))
            .unwrap_or_else(|e| {
                error!(
                    "Invalid genesis provided by Tendermint.\n{}\nTerminating process",
                    e.to_string()
                );
                std::process::exit(1)
            });

        Bank::init_genesis(&mut ctx.as_any(), genesis.bank);
        Auth::init_genesis(&mut ctx.as_any(), genesis.auth);

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
                log: "not implemented".to_string(),
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

                    let res = Bank::query_all_balances(&ctx, req).encode_vec();

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

                    // match res {
                    //     Ok(res) => {
                    //         let res = res.encode_vec();

                    //         ResponseQuery {
                    //             code: 0,
                    //             log: "exists".to_string(),
                    //             info: "".to_string(),
                    //             index: 0,
                    //             key: request.data,
                    //             value: res.into(),
                    //             proof_ops: None,
                    //             height: self
                    //                 .get_block_height()
                    //                 .try_into()
                    //                 .expect("can't believe we made it this far"),
                    //             codespace: "".to_string(),
                    //         }
                    //     }
                    //     Err(e) => ResponseQuery {
                    //         code: 0,
                    //         log: e.to_string(),
                    //         info: "".to_string(),
                    //         index: 0,
                    //         key: request.data,
                    //         value: vec![].into(),
                    //         proof_ops: None,
                    //         height: self
                    //             .get_block_height()
                    //             .try_into()
                    //             .expect("can't believe we made it this far"),
                    //         codespace: "".to_string(),
                    //     },
                    // }
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
            Ok(events) => ResponseDeliverTx {
                code: 0,
                data: Default::default(),
                log: "".to_string(),
                info: "".to_string(),
                gas_wanted: 0,
                gas_used: 0,
                events: events.into_iter().map(|e| e.into()).collect(),
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

    fn begin_block(&self, request: RequestBeginBlock) -> ResponseBeginBlock {
        info!("Got begin block request");

        self.set_block_header(
            request
                .header
                .expect("tendermint will never send nothing to the app")
                .try_into()
                .expect("tendermint will send a valid Header struct"),
        );

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
