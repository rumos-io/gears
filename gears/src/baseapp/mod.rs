pub mod options;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{atomic::AtomicU64, Arc, RwLock},
};

use crate::types::tx::TxMessage;
use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    error::{AppError, POISONED_LOCK},
    params::ParamsSubspaceKey,
    types::{
        context::{query::QueryContext, simple::SimpleContext},
        gas::{descriptor::BLOCK_GAS_DESCRIPTOR, FiniteGas, Gas},
        header::Header,
        tx::raw::TxWithRaw,
    },
};
use bytes::Bytes;
use database::RocksDB;
use store_crate::{
    types::{multi::MultiBank, query::QueryMultiStore},
    ApplicationStore,
};
use tendermint::types::{
    chain_id::ChainIdErrors,
    proto::{event::Event, header::RawHeader},
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
    block_header: Arc<RwLock<Option<RawHeader>>>, // passed by Tendermint in call to begin_block
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

        let ctx = SimpleContext::new(&mut multi_store);

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
        APP_HEIGHT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    fn get_block_header(&self) -> Option<RawHeader> {
        self.block_header
            .read()
            .expect("RwLock will not be poisoned")
            .clone()
    }

    fn set_block_header(&self, header: RawHeader) {
        let mut current_header = self
            .block_header
            .write()
            .expect("RwLock will not be poisoned");
        *current_header = Some(header);
    }

    fn get_last_commit_hash(&self) -> [u8; 32] {
        self.multi_store
            .write()
            .expect(POISONED_LOCK)
            .head_commit_hash()
    }

    fn run_query(&self, request: &RequestQuery) -> Result<Bytes, AppError> {
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
        let header: Header = self
            .get_block_header()
            .expect("block header is set in begin block")
            .try_into()
            .map_err(|e: ChainIdErrors| RunTxError::Custom(e.to_string()))?;

        let consensus_params = {
            let multi_store = &mut *self.multi_store.write().expect(POISONED_LOCK);
            let ctx = SimpleContext::new(multi_store);
            self.baseapp_params_keeper.consensus_params(&ctx)
        };

        let mut ctx = mode.build_ctx(
            height,
            header.clone(),
            consensus_params,
            Some(&tx_with_raw.tx.auth_info.fee),
            self.options.clone(),
        );

        let mut multi_store = self.multi_store.write().expect(POISONED_LOCK);

        MD::runnable(&mut ctx)?;
        MD::run_ante_checks(&mut ctx, &self.abci_handler, &tx_with_raw)?;

        let gas_wanted = ctx.gas_meter.limit(); // TODO its needed for gas recovery middleware
        let gas_used = ctx.gas_meter.consumed_or_limit();

        let events = MD::run_msg(
            &mut ctx,
            &self.abci_handler,
            tx_with_raw.tx.get_msgs().iter(),
        )?;

        ctx.block_gas_meter
            .consume_gas(gas_used, BLOCK_GAS_DESCRIPTOR)?;

        MD::commit(ctx, &mut multi_store);

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
