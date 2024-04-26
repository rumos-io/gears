use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    error::{AppError, POISONED_LOCK},
    params::{Keeper, ParamsSubspaceKey},
    types::{
        context::query::QueryContext,
        gas::{descriptor::BLOCK_GAS_DESCRIPTOR, Gas},
        header::Header,
        tx::{raw::TxWithRaw, TxMessage},
    },
};
use bytes::Bytes;
use store_crate::{database::RocksDB, types::multi::MultiStore, QueryableMultiKVStore, StoreKey};
use tendermint::types::{
    chain_id::ChainIdErrors,
    proto::{event::Event, header::RawHeader},
    request::query::RequestQuery,
};

use self::{
    errors::RunTxError,
    genesis::Genesis,
    mode::{check::CheckTxMode, deliver::DeliverTxMode, ExecutionMode},
    params::BaseAppParamsKeeper,
};

mod abci;
pub mod errors;
pub mod genesis;
pub mod mode;
mod params;

#[derive(Debug, Clone)]
pub struct BaseApp<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: TxMessage,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
> {
    height: Arc<RwLock<u64>>,
    check_mode: Arc<RwLock<CheckTxMode<RocksDB, SK>>>,
    deliver_mode: Arc<RwLock<DeliverTxMode<RocksDB, SK>>>,
    abci_handler: H,
    block_header: Arc<RwLock<Option<RawHeader>>>, // passed by Tendermint in call to begin_block
    baseapp_params_keeper: BaseAppParamsKeeper<SK, PSK>,
    pub m: PhantomData<M>,
    pub g: PhantomData<G>,
    _info_marker: PhantomData<AI>,
}

impl<
        M: TxMessage,
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        H: ABCIHandler<M, SK, G>,
        G: Genesis,
        AI: ApplicationInfo,
    > BaseApp<SK, PSK, M, H, G, AI>
{
    pub fn new(
        db: RocksDB,
        params_keeper: Keeper<SK, PSK>,
        params_subspace_key: PSK,
        abci_handler: H,
    ) -> Self {
        let db = Arc::new(db);

        let multi_store = MultiStore::new(Arc::clone(&db));
        let baseapp_params_keeper = BaseAppParamsKeeper {
            params_keeper,
            params_subspace_key,
        };

        let max_gas = baseapp_params_keeper
            .block_params(&multi_store)
            .map(|e| e.max_gas)
            .unwrap_or_default();

        let height = multi_store.head_version().into();
        Self {
            abci_handler,
            block_header: Arc::new(RwLock::new(None)),
            baseapp_params_keeper,
            height: Arc::new(RwLock::new(height)),
            check_mode: Arc::new(RwLock::new(CheckTxMode::new(
                MultiStore::new(db),
                Gas::new(max_gas),
            ))),
            deliver_mode: Arc::new(RwLock::new(DeliverTxMode::new(
                multi_store,
                Gas::new(max_gas),
            ))),
            m: PhantomData,
            g: PhantomData,
            _info_marker: PhantomData,
        }
    }

    pub fn get_block_height(&self) -> u64 {
        *self.height.read().expect("RwLock will not be poisoned")
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
        self.deliver_mode
            .read()
            .expect(POISONED_LOCK)
            .multi_store
            .head_commit_hash()
    }

    fn increment_block_height(&self) -> u64 {
        let mut height = self.height.write().expect("RwLock will not be poisoned");
        *height += 1;
        *height
    }

    fn run_query(&self, request: &RequestQuery) -> Result<Bytes, AppError> {
        let version: u32 = request.height.try_into().map_err(|_| {
            AppError::InvalidRequest("Block height must be greater than or equal to zero".into())
        })?;

        let mode = self.deliver_mode.read().expect(POISONED_LOCK);
        let ctx = QueryContext::new(&mode.multi_store, version)?;

        self.abci_handler.query(&ctx, request.clone())
    }

    fn validate_basic_tx_msgs(msgs: &Vec<M>) -> Result<(), AppError> {
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

    fn run_tx<MD: ExecutionMode<RocksDB, SK>>(
        &self,
        raw: Bytes,
        mode: &mut MD,
    ) -> Result<Vec<Event>, RunTxError> {
        let tx_with_raw: TxWithRaw<M> = TxWithRaw::from_bytes(raw.clone())
            .map_err(|e: core_types::errors::Error| RunTxError::TxParseError(e.to_string()))?;

        Self::validate_basic_tx_msgs(tx_with_raw.tx.get_msgs())
            .map_err(|e| RunTxError::Validation(e.to_string()))?;

        let height = self.get_block_height();
        let header: Header = self
            .get_block_header()
            .expect("block header is set in begin block")
            .try_into()
            .map_err(|e: ChainIdErrors| RunTxError::Custom(e.to_string()))?;

        let mut ctx = mode.build_ctx(height, &header, &tx_with_raw);

        MD::runnable(&mut ctx)?;
        MD::run_ante_checks(&mut ctx, &self.abci_handler, &tx_with_raw)?;
        // let gas_wanted = ctx.gas_meter.limit(); // TODO its needed for gas recovery middleware
        let gas_used = ctx.gas_meter.consumed_or_limit();

        let mut ctx = mode.build_ctx(height, &header, &tx_with_raw);

        let events = MD::run_msg(
            &mut ctx,
            &self.abci_handler,
            tx_with_raw.tx.get_msgs().iter(),
        )?;

        ctx.block_gas_meter
            .consume_gas(gas_used, BLOCK_GAS_DESCRIPTOR)?;

        Ok(events)
    }
}
