pub mod errors;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    error::AppError,
    params::{Keeper, ParamsSubspaceKey},
    types::{
        context::{
            query_context::QueryContext,
            tx::{mode::ExecutionMode, TxContext},
        },
        gas::{
            basic_meter::BasicGasMeter,
            infinite_meter::InfiniteGasMeter,
            {Gas, GasMeter},
        },
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

use self::{errors::RunTxError, genesis::Genesis, params::BaseAppParamsKeeper};

mod abci;
pub mod genesis;
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
    multi_store: Arc<RwLock<MultiStore<RocksDB, SK>>>,
    height: Arc<RwLock<u64>>,
    block_gas_meter: Arc<RwLock<Box<dyn GasMeter>>>,
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
        let multi_store = MultiStore::new(db);
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
            multi_store: Arc::new(RwLock::new(multi_store)),
            abci_handler,
            block_header: Arc::new(RwLock::new(None)),
            baseapp_params_keeper,
            height: Arc::new(RwLock::new(height)),
            m: PhantomData,
            g: PhantomData,
            _info_marker: PhantomData,
            block_gas_meter: match max_gas > 0 {
                true => Arc::new(RwLock::new(Box::new(InfiniteGasMeter::default()))),
                false => Arc::new(RwLock::new(Box::new(BasicGasMeter::new(Gas::new(max_gas))))),
            },
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
        self.multi_store
            .read()
            .expect("RwLock will not be poisoned")
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

        let multi_store = self
            .multi_store
            .read()
            .expect("RwLock will not be poisoned");
        let ctx = QueryContext::new(&multi_store, version)?;

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

    fn run_tx<MD: ExecutionMode>(
        &self,
        raw: Bytes,
        mut mode: MD,
    ) -> Result<Vec<Event>, RunTxError> {
        mode.runnable()?;

        let tx_with_raw: TxWithRaw<M> = TxWithRaw::from_bytes(raw.clone())
            .map_err(|e: core_types::errors::Error| RunTxError::TxParseError(e.to_string()))?;

        Self::validate_basic_tx_msgs(tx_with_raw.tx.get_msgs())
            .map_err(|e| RunTxError::Validation(e.to_string()))?;

        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");

        let mut ctx: TxContext<'_, _, _> = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block")
                .try_into()
                .map_err(|e: ChainIdErrors| RunTxError::Custom(e.to_string()))?,
        );

        mode.run_ante_checks(&mut ctx, &self.abci_handler, &tx_with_raw)?;

        let mut ctx: TxContext<'_, _, _> = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block")
                .try_into()
                .map_err(|e: ChainIdErrors| RunTxError::Custom(e.to_string()))?,
        );

        let events = mode.run_msg(
            &mut ctx,
            &self.abci_handler,
            tx_with_raw.tx.get_msgs().iter(),
        )?;

        Ok(events)
    }
}
