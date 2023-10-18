use bytes::Bytes;
use database::{Database, RocksDB};
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
        context::{Context, ExecMode},
        init_context::InitContext,
        query_context::QueryContext,
        tx_context::TxContext,
    },
    mempool::MemPool,
};
use crate::{
    error::AppError,
    x::params::{Keeper, ParamsSubspaceKey},
};

use super::{
    ante::{AnteHandler, AuthKeeper, BankKeeper},
    errors::{TxValidationError, RunTxError},
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

    // fn cache_tx_context<'a, T: Database>(
    //     &'a self,
    //     ctx: &'a mut Context<T, SK>,
    //     _tx_bytes: &Bytes,
    // ) -> (Context<T, SK>, CacheMS) {
    //     let ms_cache = ctx.multi_store().cache_multi_store();

    //     if ms_cache.is_tracing_enabled() {
    //         ms_cache.tracing_context_set();
    //     }

    //     (ctx.with_multi_store( /* ms_cache */ ), ms_cache)
    // }

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
    ) -> Result<Vec<tendermint_informal::abci::Event>, RunTxError> {
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

        let mut ctx = Context::TxContext(&mut inner_ctx);

        match self.base_ante_handler.run(&mut ctx, &tx_with_raw) {
            Ok(_) => ctx.multi_store_mut().write_then_clear_tx_caches(),
            Err(e) => {
                ctx.multi_store_mut().clear_tx_caches();
                return Err(RunTxError::AnteError(e));
            }
        }; // TODO I don't like how this block looks

        // only run the tx if there is block gas remaining
        if mode == ExecMode::ModeFinalize && ctx.block_gas_meter().is_out_of_gas() {
            // return gInfo, nil, nil, errorsmod.Wrap(sdkerrors.ErrOutOfGas, "no block gas left to run tx")
            todo!()
        }
        // https://github.com/cosmos/cosmos-sdk/blob/faca642586821f52e3492f6f1cdf044034afcbcc/baseapp/baseapp.go#L812
        // TODO

        // let events = match self.run_msgs(&mut ctx, tx_with_raw.tx.get_msgs()) {
        //     Ok(_) => {
        //         let events = ctx.events.clone(); //TODO: Think how to remove clone
        //         multi_store_lock.write_then_clear_tx_caches();
        //         Ok(events)
        //     }
        //     Err(e) => {
        //         multi_store_lock.clear_tx_caches();
        //         Err(e)
        //     }
        // };

        let _gas_wanted = {
            // 863 - 901
            // let mut ctx_innter = (&mut ctx).into(); // fixes drop tmp value issue

            // let (mut ante_ctx, ms_cache) = self.cache_tx_context(&mut ctx_innter, &raw);
            // ante_ctx.event_manager_set(EventManager); // In cosmos sdk it returns new pointer with empty event

            // newCtx, err := app.anteHandler(anteCtx, tx, mode == execModeSimulate)

            // ctx = newCtx.WithMultiStore(ms)

            // let events = ctx_innter.events;
            // GasMeter expected to be set in AnteHandler
            // let gas_wanted = ctx_innter.gas_meter().limit();

            // ms_cache.write();
            // anteEvents = events.ToABCIEvents()

            // gas_wanted
            1
        };

        // 903 - 914
        // if mode == ExecMode::Check {
        //     let ctx_inner = (&mut ctx).into();

        //     if let Err(_val) = self.mempool.insert_tx(&ctx_inner, &tx_with_raw.tx) {
        //         // return gInfo, nil, anteEvents, err
        //     }
        // } else if mode == ExecMode::ModeFinalize {
        //     if let Err(_val) = self.mempool.remove_tx(&tx_with_raw.tx) {
        //         // if var == ExNotFound...
        //         // return gInfo, nil, anteEvents, fmt.Errorf("failed to remove tx from mempool: %w", err)
        //     }
        // }

        // Create a new Context based off of the existing Context with a MultiStore branch
        // in case message processing fails. At this point, the MultiStore
        // is a branch of a branch.
        {
            // let mut ctx_inner = (&mut ctx).into(); // fixes drop tmp value issue

            // let (mut _ante_ctx, ms_cache) = self.cache_tx_context(&mut ctx_inner, &raw);

            // let _msg = tx_with_raw.tx.get_msgs(); // TODO: Cosmos uses v2

            // if app.postHandler != nil {
            //     // The runMsgCtx context currently contains events emitted by the ante handler.
            //     // We clear this to correctly order events without duplicates.
            //     // Note that the state is still preserved.
            //     postCtx := runMsgCtx.WithEventManager(sdk.NewEventManager())

            //     newCtx, err := app.postHandler(postCtx, tx, mode == execModeSimulate, err == nil)
            //     if err != nil {
            //         return gInfo, nil, anteEvents, err
            //     }

            //     result.Events = append(result.Events, newCtx.EventManager().ABCIEvents()...)
            // }

            // https://github.com/cosmos/cosmos-sdk/blob/faca642586821f52e3492f6f1cdf044034afcbcc/baseapp/baseapp.go#L808C3-L808C3
            if mode == ExecMode::ModeFinalize {
                // ctx_inner
                // .block_gas_meter_mut()
                // .consume_gas
                // (
                //     ctx.gas_meter().gas_consumed_to_limit(), "block gas meter".to_string(),
                // );

                // ms_cache.write();

                // TODO: borrow issue
            }

            // if len(anteEvents) > 0 && (mode == execModeFinalize || mode == execModeSimulate) {
            // 	// append the events in the order of occurrence
            // 	result.Events = append(anteEvents, result.Events...)
            // }
        }

        Ok(Vec::new())
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
