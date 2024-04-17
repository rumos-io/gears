pub mod errors;
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    error::AppError,
    params::{Keeper, ParamsSubspaceKey},
    types::{
        context::{
            query_context::QueryContext, tx_context::TxContext, ContextOptions, ExecMode,
            TransactionalContext,
        },
        tx::{raw::TxWithRaw, TxMessage},
    },
};
use bytes::Bytes;
use store_crate::TransactionalMultiKVStore;
use store_crate::{
    database::{Database, RocksDB},
    types::multi::MultiStore,
    QueryableMultiKVStore, StoreKey,
};
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
    ctx_options: Arc<RwLock<ContextOptions>>,
    height: Arc<RwLock<u64>>,
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
            ctx_options: Default::default(),
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

    fn run_tx(&self, raw: Bytes, mode: ExecMode) -> Result<Vec<Event>, RunTxError> {
        let tx_with_raw: TxWithRaw<M> = TxWithRaw::from_bytes(raw.clone())
            .map_err(|e: core_types::errors::Error| RunTxError::TxParseError(e.to_string()))?;

        Self::validate_basic_tx_msgs(tx_with_raw.tx.get_msgs())
            .map_err(|e| RunTxError::Validation(e.to_string()))?;

        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");

        // TODO: Constructor
        let mut ctx = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block")
                .try_into()
                .map_err(|e: ChainIdErrors| RunTxError::Custom(e.to_string()))?,
            raw.clone().into(),
        );

        match self.abci_handler.run_ante_checks(&mut ctx, &tx_with_raw) {
            Ok(_) => {
                if mode != ExecMode::Deliver {
                    multi_store.tx_caches_clear();
                } else {
                    multi_store.tx_caches_write_then_clear()
                }
            }
            Err(e) => {
                multi_store.tx_caches_clear();
                return Err(RunTxError::Custom(e.to_string()));
            }
        };

        let mut ctx = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block")
                .try_into()
                .map_err(|e: ChainIdErrors| RunTxError::Custom(e.to_string()))?,
            raw.clone().into(),
        );

        let msg_run = match self.run_msgs(&mut ctx, tx_with_raw.tx.get_msgs(), mode.clone()) {
            Ok(_) => {
                // ctx.gas_meter_mut()
                //     .consume_to_limit()
                //     .map_err(|e| RunTxError::Custom(e.to_string()))?;

                let events = ctx.events;
                if mode != ExecMode::Deliver {
                    multi_store.tx_caches_clear();
                } else {
                    multi_store.tx_caches_write_then_clear()
                }
                Ok(events)
            }
            Err(e) => {
                multi_store.tx_caches_clear();
                Err(e)
            }
        }
        .map_err(|e| RunTxError::Custom(e.to_string()))?;

        Ok(msg_run)
    }

    fn run_msgs<T: Database + Sync + Send>(
        &self,
        ctx: &mut impl TransactionalContext<T, SK>,
        msgs: &Vec<M>,
        mode: ExecMode,
    ) -> Result<(), AppError> {
        for msg in msgs {
            if mode == ExecMode::Check || mode == ExecMode::ReCheck {
                break;
            }

            self.abci_handler.tx(ctx, msg)?
        }

        Ok(())
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
}
