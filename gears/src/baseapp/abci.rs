use super::BaseApp;
use crate::application::handlers::node::ABCIHandler;
use crate::application::ApplicationInfo;
use crate::baseapp::RunTxInfo;
use crate::context::{block::BlockContext, init::InitContext};
use crate::error::POISONED_LOCK;
use crate::params::ParamsSubspaceKey;
use crate::types::gas::Gas;
use bytes::Bytes;
use tendermint::{
    application::ABCIApplication,
    types::{
        request::{
            begin_block::RequestBeginBlock,
            check_tx::RequestCheckTx,
            deliver_tx::RequestDeliverTx,
            echo::RequestEcho,
            end_block::RequestEndBlock,
            info::RequestInfo,
            init_chain::RequestInitChain,
            query::RequestQuery,
            snapshot::{RequestApplySnapshotChunk, RequestLoadSnapshotChunk, RequestOfferSnapshot},
        },
        response::{
            begin_block::ResponseBeginBlock,
            check_tx::ResponseCheckTx,
            deliver_tx::ResponseDeliverTx,
            echo::ResponseEcho,
            end_block::ResponseEndBlock,
            info::ResponseInfo,
            init_chain::ResponseInitChain,
            query::ResponseQuery,
            snapshot::{
                ResponseApplySnapshotChunk, ResponseListSnapshots, ResponseLoadSnapshotChunk,
                ResponseOfferSnapshot,
            },
            ResponseCommit, ResponseFlush,
        },
    },
};
use tracing::{debug, error, info};

impl<PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo> ABCIApplication<H::Genesis>
    for BaseApp<PSK, H, AI>
{
    fn init_chain(&self, request: RequestInitChain<H::Genesis>) -> ResponseInitChain {
        info!("Got init chain request");

        let mut multi_store = self.multi_store.write().expect(POISONED_LOCK);

        //TODO: handle request height > 1 as is done in SDK

        let mut ctx = InitContext::new(
            &mut multi_store,
            self.block_height(),
            request.time,
            request.chain_id,
        );

        self.baseapp_params_keeper
            .set_consensus_params(&mut ctx, request.consensus_params.clone());

        self.abci_handler
            .init_genesis(&mut ctx, request.app_genesis.clone());

        ResponseInitChain {
            consensus_params: Some(request.consensus_params),
            validators: request.validators,
            app_hash: "hash_goes_here".into(), //TODO: set app hash
        }
    }

    fn info(&self, request: RequestInfo) -> ResponseInfo {
        info!(
            "Got info request. Tendermint version: {}; Block version: {}; P2P version: {}",
            request.version, request.block_version, request.p2p_version
        );

        ResponseInfo {
            data: AI::APP_NAME.to_owned(),
            version: AI::APP_VERSION.to_owned(),
            app_version: 1,
            last_block_height: self
                .block_height()
                .try_into()
                .expect("can't believe we made it this far"),
            last_block_app_hash: self.get_last_commit_hash().to_vec().into(),
        }
    }

    fn query(&self, request: RequestQuery) -> ResponseQuery {
        info!("Got query request to: {}", request.path);

        match self.run_query(&request) {
            Ok(res) => ResponseQuery {
                code: 0,
                log: "exists".to_string(),
                info: "".to_string(),
                index: 0,
                key: request.data,
                value: res,
                proof_ops: None,
                height: self
                    .block_height()
                    .try_into()
                    .expect("can't believe we made it this far"),
                codespace: "".to_string(),
            },
            Err(e) => ResponseQuery {
                code: 1,
                log: e.to_string(),
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

    fn check_tx(&self, RequestCheckTx { tx, r#type }: RequestCheckTx) -> ResponseCheckTx {
        info!("Got check tx request");

        let mut state = self.state.write().expect(POISONED_LOCK);

        let result = match r#type {
            0 | 1 => self.run_tx(tx.clone(), &mut state.check_mode), // TODO: ReCheckTxMode
            _ => panic!("unknown Request CheckTx type: {}", r#type),
        };

        match result {
            Ok(RunTxInfo {
                events,
                gas_wanted,
                gas_used,
            }) => {
                debug!("{:?}", events);
                ResponseCheckTx {
                    code: 0,
                    data: Default::default(),
                    log: "".to_string(),
                    info: "".to_string(),
                    gas_wanted: gas_wanted.into(),
                    gas_used: gas_used.into(),
                    events,
                    codespace: "".to_string(),
                    mempool_error: "".to_string(),
                    priority: 0,
                    sender: "".to_string(),
                }
            }
            Err(e) => {
                error!("check err: {e}");
                ResponseCheckTx {
                    code: 1,
                    data: Default::default(),
                    log: e.to_string(),
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
        }
    }

    fn deliver_tx(&self, RequestDeliverTx { tx }: RequestDeliverTx) -> ResponseDeliverTx {
        info!("Got deliver tx request");

        let mut state = self.state.write().expect(POISONED_LOCK);

        let result = self.run_tx(tx.clone(), &mut state.deliver_mode);

        match result {
            Ok(RunTxInfo {
                events,
                gas_wanted,
                gas_used,
            }) => ResponseDeliverTx {
                code: 0,
                data: Default::default(),
                log: "".to_string(),
                info: "".to_string(),
                gas_wanted: gas_wanted.into(),
                gas_used: gas_used.into(),
                events: events.into_iter().collect(),
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

        let height = self.block_height();

        let hash = self.multi_store.write().expect(POISONED_LOCK).commit();

        info!(
            "Committed state, block height: {} app hash: {}",
            height,
            hex::encode(hash)
        );

        self.state.write().expect(POISONED_LOCK).cache_clear();

        ResponseCommit {
            data: hash.to_vec().into(),
            retain_height: (height - 1)
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

        self.block_height_increment();

        self.set_block_header(request.header.clone());

        let mut multi_store = self.multi_store.write().expect(POISONED_LOCK);
        let mut state = self.state.write().expect(POISONED_LOCK);

        let mut ctx = BlockContext::new(
            &mut multi_store,
            self.block_height(),
            request.header.clone(),
        );

        {
            let max_gas = self
                .baseapp_params_keeper
                .block_params(&mut ctx)
                .map(|e| e.max_gas)
                .unwrap_or_default(); // This is how cosmos handles it  https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/baseapp/baseapp.go#L497

            state.replace_meter(Gas::from(max_gas))
        }

        self.abci_handler.begin_block(&mut ctx, request);

        let events = ctx.events;

        state.cache_update(&mut multi_store);

        ResponseBeginBlock {
            events: events.into_iter().collect(),
        }
    }

    fn end_block(&self, request: RequestEndBlock) -> ResponseEndBlock {
        info!("Got end block request");

        let mut multi_store = self.multi_store.write().expect(POISONED_LOCK);

        let mut ctx = BlockContext::new(
            &mut multi_store,
            self.block_height(),
            self.get_block_header()
                .expect("block header is set in begin block"), //TODO: return error?
        );

        let validator_updates = self.abci_handler.end_block(&mut ctx, request);

        let events = ctx.events;

        self.state
            .write()
            .expect(POISONED_LOCK)
            .cache_update(&mut multi_store);

        ResponseEndBlock {
            events: events.into_iter().collect(),
            validator_updates,
            consensus_param_updates: None,
            // TODO: there is only one call to BaseAppParamsKeeper::set_consensus_params,
            // which is made during init. This means that these params cannot change.
            // However a get method should be implemented in future.
        }
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
