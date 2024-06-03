pub mod options;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{atomic::AtomicU64, Arc, RwLock},
};

use crate::types::tx::TxMessage;
use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    context::{query::QueryContext, simple::SimpleContext},
    error::{AppError, POISONED_LOCK},
    params::ParamsSubspaceKey,
    types::{
        gas::{descriptor::BLOCK_GAS_DESCRIPTOR, FiniteGas, Gas},
        tx::raw::TxWithRaw,
    },
};
use bytes::Bytes;
use database::RocksDB;
use kv_store::{
    types::{multi::MultiBank, query::QueryMultiStore},
    ApplicationStore,
};
use tendermint::types::{
    proto::{event::Event, header::Header},
    request::query::RequestQuery,
};

use self::{
    errors::RunTxError, mode::ExecutionMode, options::NodeOptions, params::BaseAppParamsKeeper,
    state::ApplicationState,
};

mod abci;
pub mod errors;
pub mod genesis;
pub mod mode;
mod params;
mod query;
pub mod state;
pub use params::{BlockParams, ConsensusParams, EvidenceParams, ValidatorParams};

pub use query::*;

static APP_HEIGHT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct BaseApp<PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo> {
    state: Arc<RwLock<ApplicationState<RocksDB, H>>>,
    multi_store: Arc<RwLock<MultiBank<RocksDB, H::StoreKey, ApplicationStore>>>,
    abci_handler: H,
    block_header: Arc<RwLock<Option<Header>>>, // passed by Tendermint in call to begin_block
    baseapp_params_keeper: BaseAppParamsKeeper<PSK>,
    options: NodeOptions,
    _info_marker: PhantomData<AI>,
}

impl<PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo> BaseApp<PSK, H, AI> {
    pub fn new(
        db: RocksDB,
        params_subspace_key: PSK,
        abci_handler: H,
        options: NodeOptions,
    ) -> Self {
        let mut multi_store = MultiBank::<_, _, ApplicationStore>::new(db);

        let baseapp_params_keeper = BaseAppParamsKeeper {
            params_subspace_key,
        };

        let ctx = SimpleContext::from(&mut multi_store);

        let max_gas = baseapp_params_keeper
            .block_params(&ctx)
            .map(|e| e.max_gas)
            .unwrap_or_default();

        block_height_set(multi_store.head_version() as u64);

        // For now let this func to exists only in new method
        fn block_height_set(height: u64) {
            let _ = APP_HEIGHT.swap(height, std::sync::atomic::Ordering::Relaxed);
        }

        Self {
            abci_handler,
            block_header: Arc::new(RwLock::new(None)),
            baseapp_params_keeper,
            state: Arc::new(RwLock::new(ApplicationState::new(
                Gas::from(max_gas),
                &multi_store,
            ))),
            multi_store: Arc::new(RwLock::new(multi_store)),
            options,
            _info_marker: PhantomData,
        }
    }

    fn block_height(&self) -> u64 {
        APP_HEIGHT.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn block_height_increment(&self) -> u64 {
        APP_HEIGHT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1 //TODO: wraps on overflow
    }

    fn get_block_header(&self) -> Option<Header> {
        self.block_header.read().expect(POISONED_LOCK).clone()
    }

    fn set_block_header(&self, header: Header) {
        let mut current_header = self.block_header.write().expect(POISONED_LOCK);
        *current_header = Some(header);
    }

    fn get_last_commit_hash(&self) -> [u8; 32] {
        self.multi_store
            .write()
            .expect(POISONED_LOCK)
            .head_commit_hash()
    }

    fn run_query(&self, request: &RequestQuery) -> Result<Bytes, AppError> {
        //TODO: request height u32
        let version: u32 = request.height.try_into().map_err(|_| {
            AppError::InvalidRequest("Block height must be greater than or equal to zero".into())
        })?;

        let query_store =
            QueryMultiStore::new(&*self.multi_store.read().expect(POISONED_LOCK), version)?;

        let ctx = QueryContext::new(query_store, version)?;

        self.abci_handler.query(&ctx, request.clone())
    }

    fn validate_basic_tx_msgs(msgs: &Vec<H::Message>) -> Result<(), AppError> {
        if msgs.is_empty() {
            return Err(AppError::InvalidRequest(
                "must contain at least one message".into(),
            ));
        }

        for msg in msgs {
            msg.validate_basic()
                .map_err(|e| AppError::TxValidation(e.to_string()))?
        }

        Ok(())
    }

    fn run_tx<MD: ExecutionMode<RocksDB, H>>(
        &self,
        raw: Bytes,
        mode: &mut MD,
    ) -> Result<RunTxInfo, RunTxError> {
        let tx_with_raw: TxWithRaw<H::Message> = TxWithRaw::from_bytes(raw.clone())
            .map_err(|e: core_types::errors::Error| RunTxError::TxParseError(e.to_string()))?;

        Self::validate_basic_tx_msgs(tx_with_raw.tx.get_msgs())
            .map_err(|e| RunTxError::Validation(e.to_string()))?;

        let height = self.block_height();

        let consensus_params = {
            let ctx: SimpleContext<RocksDB, <H as ABCIHandler>::StoreKey> =
                SimpleContext::from(mode.multi_store());
            self.baseapp_params_keeper.consensus_params(&ctx)
        };

        let mut ctx = mode.build_ctx(
            height,
            self.get_block_header()
                .expect("block header is set in begin block"), //TODO: return error
            consensus_params,
            Some(&tx_with_raw.tx.auth_info.fee),
            self.options.clone(),
        );

        MD::runnable(&mut ctx)?;
        MD::run_ante_checks(&mut ctx, &self.abci_handler, &tx_with_raw)?;

        let gas_wanted = ctx.gas_meter.borrow().limit();
        let gas_used = ctx.gas_meter.borrow().consumed_or_limit();

        let events = MD::run_msg(
            &mut ctx,
            &self.abci_handler,
            tx_with_raw.tx.get_msgs().iter(),
        )?;

        ctx.block_gas_meter
            .consume_gas(gas_used, BLOCK_GAS_DESCRIPTOR)?;

        std::mem::drop(ctx);

        let mut multi_store = self.multi_store.write().expect(POISONED_LOCK);
        mode.commit(&mut multi_store);

        Ok(RunTxInfo {
            events,
            gas_wanted,
            gas_used,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RunTxInfo {
    pub events: Vec<Event>,
    pub gas_wanted: Gas,
    pub gas_used: FiniteGas,
}
