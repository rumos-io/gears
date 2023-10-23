use bytes::Bytes;
use database::{Database, RocksDB};
use ibc_proto::cosmos::base::abci::v1beta1::{GasInfo, Result as Product};
use ibc_relayer::util::lock::LockExt;
use proto_messages::cosmos::{
    base::v1beta1::SendCoins,
    tx::v1beta1::{Message, TxWithRaw},
};
use proto_types::AccAddress;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};
use store_crate::{MultiStore, StoreKey};
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

use crate::types::{
    context::{
        context::{Context, ContextTrait, EventManager, ExecMode},
        init_context::InitContext,
        query_context::QueryContext,
        tx_context::TxContext,
    },
    mempool::{MemPool, MempoolTrait},
};
use crate::{
    error::AppError,
    x::params::{Keeper, ParamsSubspaceKey},
};

use super::{
    ante::{AnteHandler, AuthKeeper, BankKeeper},
    errors::{RunTxError, TxValidationError},
    params::BaseAppParamsKeeper,
};

pub trait Handler<M: Message, SK: StoreKey, G: DeserializeOwned + Clone + Send + Sync + 'static>:
    Clone + Send + Sync + 'static
{
    fn handle_tx<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &M,
    ) -> Result<(), AppError>;

    fn handle_begin_block<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        request: RequestBeginBlock,
    );

    fn handle_init_genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: G);

    fn handle_query<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        query: RequestQuery,
    ) -> Result<Bytes, AppError>;

    fn handle_add_genesis_account(
        &self,
        genesis_state: &mut G,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError>;
}

pub trait Genesis: DeserializeOwned + Serialize + Clone + Send + Sync + 'static {}
impl<T: DeserializeOwned + Serialize + Clone + Send + Sync + 'static> Genesis for T {}

#[allow(dead_code)] //TODO: Remove
#[derive(Debug, Clone)]
pub struct BaseApp<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    H: Handler<M, SK, G>,
    G: Genesis,
> {
    multi_store: Arc<RwLock<MultiStore<RocksDB, SK>>>,
    height: Arc<RwLock<u64>>,
    base_ante_handler: AnteHandler<BK, AK, SK>,
    handler: H,
    block_header: Arc<RwLock<Option<Header>>>, // passed by Tendermint in call to begin_block
    baseapp_params_keeper: BaseAppParamsKeeper<SK, PSK>,
    app_name: &'static str,
    app_version: &'static str,
    mempool: MemPool,
    pub m: PhantomData<M>,
    pub g: PhantomData<G>,
}

impl<
        M: Message,
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        BK: BankKeeper<SK>,
        AK: AuthKeeper<SK>,
        H: Handler<M, SK, G>,
        G: Genesis,
    > Application for BaseApp<SK, PSK, M, BK, AK, H, G>
{
    fn init_chain(&self, request: RequestInitChain) -> ResponseInitChain {
        info!("Got init chain request");
        let mut multi_store = self.multi_store.acquire_write();

        //TODO: handle request height > 1 as is done in SDK

        let mut ctx = InitContext::new(&mut multi_store, self.get_block_height(), request.chain_id);

        if let Some(params) = request.consensus_params.clone() {
            self.baseapp_params_keeper
                .set_consensus_params(&mut (&mut ctx).into(), params);
        }

        let genesis: G = String::from_utf8(request.app_state_bytes.into())
            .map_err(|e| AppError::Genesis(e.to_string()))
            .and_then(|s| serde_json::from_str(&s).map_err(|e| AppError::Genesis(e.to_string())))
            .unwrap_or_else(|e| {
                error!(
                    "Invalid genesis provided by Tendermint.\n{}\nTerminating process",
                    e.to_string()
                );
                std::process::exit(1)
            });

        self.handler.handle_init_genesis(&mut ctx, genesis);

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
            data: self.app_name.to_string(),
            version: self.app_version.to_string(),
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

        match self.run_query(&request) {
            Ok(res) => ResponseQuery {
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

    /// CheckTx implements the `ABCI` interface and executes a tx in CheckTx mode. In
    /// CheckTx mode, messages are not executed. This means messages are only validated
    /// and only the AnteHandler is executed. State is persisted to the BaseApp's
    /// internal CheckTx state if the AnteHandler passes. Otherwise, the ResponseCheckTx
    /// will contain relevant error information. Regardless of tx execution outcome,
    /// the ResponseCheckTx will contain relevant gas execution context.
    fn check_tx(&self, request: RequestCheckTx) -> ResponseCheckTx {
        info!("Got check tx request");

        let exec_mode = match request.r#type {
            0 => ExecMode::Check,           //NEW
            1 => ExecMode::ReCheck,         //RECHECK
            _ => return Default::default(), // TODO: Create error like or Error for this case
        };

        let result = match self.run_tx(request.tx, exec_mode) {
            Ok(_result) => ResponseCheckTx {
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
            },
            Err(_) => Default::default(), // TODO: Create error like
        };

        result
    }

    fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx {
        info!("Got deliver tx request");
        match self.run_tx(request.tx, ExecMode::ModeFinalize) {
            Ok((_gas_info, _product, events, _priority)) => ResponseDeliverTx {
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

    // #[ tracing::instrument(
    //     name = "Got echo request",
    //     skip_all, )] //TODO: Ask
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
                .clone()
                .expect("tendermint will never send nothing to the app")
                .try_into()
                .expect("tendermint will send a valid Header struct"),
        );

        let mut multi_store = self.multi_store.acquire_write();

        let mut ctx = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block"),
            vec![],
        );

        self.handler.handle_begin_block(&mut ctx, request);

        let events = ctx.events;
        multi_store.write_then_clear_tx_caches();

        ResponseBeginBlock {
            events: events.into_iter().map(|e| e.into()).collect(),
        }
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

#[allow(dead_code)] // TODO: Remove
impl<
        M: Message,
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        BK: BankKeeper<SK>,
        AK: AuthKeeper<SK>,
        H: Handler<M, SK, G>,
        G: Genesis,
    > BaseApp<SK, PSK, M, BK, AK, H, G>
{
    pub fn new(
        db: RocksDB,
        app_name: &'static str,
        version: &'static str,
        bank_keeper: BK,
        auth_keeper: AK,
        params_keeper: Keeper<SK, PSK>,
        params_subspace_key: PSK,
        handler: H,
    ) -> Self {
        let multi_store = MultiStore::new(db);
        let baseapp_params_keeper = BaseAppParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        let height = multi_store.get_head_version().into();
        Self {
            multi_store: Arc::new(RwLock::new(multi_store)),
            base_ante_handler: AnteHandler::new(bank_keeper, auth_keeper),
            handler,
            block_header: Arc::new(RwLock::new(None)),
            baseapp_params_keeper,
            height: Arc::new(RwLock::new(height)),
            app_name,
            app_version: version,
            m: PhantomData,
            g: PhantomData,
            mempool: MemPool,
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

    fn run_query(&self, request: &RequestQuery) -> Result<Bytes, AppError> {
        let version: u32 = request.height.try_into().map_err(|_| {
            AppError::InvalidRequest("Block height must be greater than or equal to zero".into())
        })?;

        let multi_store = self
            .multi_store
            .read()
            .expect("RwLock will not be poisoned");
        let ctx = QueryContext::new(&multi_store, version)?;

        self.handler.handle_query(&ctx, request.clone())
    }



    fn run_tx(
        &self,
        raw: Bytes,
        mode: ExecMode,
    ) -> Result<(GasInfo, Product, Vec<tendermint_informal::abci::Event>, Priority), RunTxError> {
        let tx_with_raw: TxWithRaw<M> = TxWithRaw::from_bytes(raw.clone())?;

        Self::validate_basic_tx_msgs(tx_with_raw.tx.get_msgs())?;

        let mut multi_store_lock = self.multi_store.acquire_write();

        let mut inner_ctx = TxContext::new(
            &mut multi_store_lock,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block"),
            raw.clone().into(),
        );

        // only run the tx if there is block gas remaining
        if mode == ExecMode::ModeFinalize && inner_ctx.block_gas_meter().is_out_of_gas() {
            return Err(RunTxError::OutOfGas);
        }

        // https://github.com/cosmos/cosmos-sdk/blob/faca642586821f52e3492f6f1cdf044034afcbcc/baseapp/baseapp.go#L812
        // TODO recovery. GO code uses recover on panic.

        let ( _gas_wanted, priority, _abci_events) = {
            let (mut ante_ctx, ms_cache) = inner_ctx.cache_tx_context(&raw);
            ante_ctx.event_manager_set(EventManager);

            let _msg = tx_with_raw.tx.get_msgs();

            // self.handler.handle_tx(&mut ante_ctx, &msg[0])
            // .map_err(|e| RunTxError::CustomError(e.to_string()))?; // should return new ctx

            // At this point, newCtx.MultiStore() is a store branch, or something else
            // replaced by the AnteHandler. We want the original multistore.
            //
            // Also, in the case of the tx aborting, we need to track gas consumed via
            // the instantiated gas meter in the AnteHandler, so we update the context prior to returning.

            // if !newCtx.IsZero() {
            //     ctx = newCtx.WithMultiStore(ms)
            // }

            // GasMeter expected to be set in AnteHandler
            let gas_wanted = ante_ctx.gas_meter().limit();

            ms_cache.write();

            (gas_wanted, Priority( ante_ctx.priority ), ante_ctx.events)
        };

        let events = {
            let (mut run_msg_ctx, _ms_cache) = inner_ctx.cache_tx_context(&raw);

            {
                let mut ctx = Context::TxContext(&mut run_msg_ctx);

                match self.base_ante_handler.run(&mut ctx, &tx_with_raw) {
                    Ok(_) => {
                        ctx.multi_store_mut().write_then_clear_tx_caches();
                        Ok(())
                    }
                    Err(e) => {
                        ctx.multi_store_mut().clear_tx_caches();
                        Err(e)
                    }
                }?
            }

            {
                match self.run_msgs(&mut run_msg_ctx, tx_with_raw.tx.get_msgs()) {
                    Ok(_) => {
                        let events = run_msg_ctx.events().clone();
                        run_msg_ctx.multi_store_mut().write_then_clear_tx_caches();
                        Ok(events)
                    }
                    Err(e) => {
                        run_msg_ctx.multi_store_mut().clear_tx_caches();
                        Err(RunTxError::CustomError(e.to_string())) //TODO proper error for ante
                    }
                }?
            }
        };

        {
            if mode == ExecMode::Check {
                let ctx = Context::TxContext(&mut inner_ctx);

                if let Err(val) = self.mempool.insert_tx(&ctx, &tx_with_raw.tx) {
                    return Err(RunTxError::CustomError(val.to_string()));
                }
            } else if mode == ExecMode::ModeFinalize {
                if let Err(_val) = self.mempool.remove_tx(&tx_with_raw.tx) {
                    // if var == ExNotFound...
                    {
                        return Err(RunTxError::CustomError(
                            "failed to remove tx from mempool".to_string(),
                        )); //TODO: mempool errors
                    }
                }
            }
        }

        if mode == ExecMode::Deliver {
            let gas = inner_ctx.gas_meter().gas_consumed_to_limit();
            inner_ctx
                .block_gas_meter_mut()
                .consume_gas(gas, "block gas meter".to_string())?;
        }

        Ok((
            GasInfo {
                gas_wanted: 1,
                gas_used: 1,
            },
            Product::default(),
            events,
            priority,
        ))
    }

    fn run_msgs<T: Database>(
        &self,
        ctx: &mut TxContext<'_, T, SK>,
        msgs: &Vec<M>,
    ) -> Result<(), AppError> {
        for msg in msgs {
            self.handler.handle_tx(ctx, msg)?
        }

        return Ok(());
    }

    fn validate_basic_tx_msgs(msgs: &Vec<M>) -> Result<(), TxValidationError> {
        if msgs.is_empty() {
            Err(TxValidationError::InvalidRequest)?
        }

        for msg in msgs {
            msg.validate_basic()
                .map_err(|e| TxValidationError::CustomError(e))?
        }

        return Ok(());
    }
}

pub struct Priority(pub i64); // TODO: Move newtype to other place