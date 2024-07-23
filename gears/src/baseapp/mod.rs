pub mod options;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    context::{query::QueryContext, simple::SimpleContext},
    error::POISONED_LOCK,
    params::ParamsSubspaceKey,
    types::{
        gas::{descriptor::BLOCK_GAS_DESCRIPTOR, FiniteGas, Gas},
        tx::raw::TxWithRaw,
    },
};
use bytes::Bytes;
use database::Database;
use errors::QueryError;
use kv_store::{
    types::{multi::MultiBank, query::QueryMultiStore},
    ApplicationStore,
};
use tendermint::types::{
    proto::{event::Event, header::Header},
    request::query::RequestQuery,
};

use self::{
    errors::RunTxError, mode::ExecutionMode, options::NodeOptions, state::ApplicationState,
};

mod abci;
pub mod errors;
pub mod genesis;
pub mod mode;
mod params;
mod query;
pub mod state;
pub use params::{
    BaseAppParamsKeeper, BlockParams, ConsensusParams, EvidenceParams, ValidatorParams,
};

pub use query::*;

#[derive(Debug, Clone)]
pub struct BaseApp<DB: Database, PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo> {
    state: Arc<RwLock<ApplicationState<DB, H>>>,
    multi_store: Arc<RwLock<MultiBank<DB, H::StoreKey, ApplicationStore>>>,
    abci_handler: H,
    block_header: Arc<RwLock<Option<Header>>>, // passed by Tendermint in call to begin_block
    baseapp_params_keeper: BaseAppParamsKeeper<PSK>,
    options: NodeOptions,
    _info_marker: PhantomData<AI>,
}

impl<DB: Database, PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo>
    BaseApp<DB, PSK, H, AI>
{
    pub fn new(db: DB, params_subspace_key: PSK, abci_handler: H, options: NodeOptions) -> Self {
        let mut multi_store = MultiBank::<_, _, ApplicationStore>::new(db);

        let baseapp_params_keeper = BaseAppParamsKeeper {
            params_subspace_key,
        };

        let height = multi_store.head_version();
        let ctx = SimpleContext::new((&mut multi_store).into(), height);

        let max_gas = baseapp_params_keeper
            .block_params(&ctx)
            .map(|e| e.max_gas)
            .unwrap_or_default();

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

    fn get_block_header(&self) -> Option<Header> {
        self.block_header.read().expect(POISONED_LOCK).clone()
    }

    fn set_block_header(&self, header: Header) {
        let mut current_header = self.block_header.write().expect(POISONED_LOCK);
        *current_header = Some(header);
    }

    fn get_last_commit_hash(&self) -> [u8; 32] {
        self.multi_store
            .read()
            .expect(POISONED_LOCK)
            .head_commit_hash()
    }

    fn get_last_commit_height(&self) -> u32 {
        self.multi_store.read().expect(POISONED_LOCK).head_version()
    }

    fn run_query(&self, request: &RequestQuery) -> Result<Bytes, QueryError> {
        //TODO: request height u32
        let version: u32 = request
            .height
            .try_into()
            .map_err(|_| QueryError::InvalidHeight)?;

        let query_store =
            QueryMultiStore::new(&*self.multi_store.read().expect(POISONED_LOCK), version)?;

        let ctx = QueryContext::new(query_store, version)?;

        self.abci_handler.query(&ctx, request.clone())
    }

    fn run_tx<MD: ExecutionMode<DB, H>>(
        &self,
        raw: Bytes,
        mode: &mut MD,
    ) -> Result<RunTxInfo, RunTxError> {
        let tx_with_raw: TxWithRaw<H::Message> =
            TxWithRaw::from_bytes(raw.clone()).map_err(|e: core_types::errors::CoreError| {
                RunTxError::InvalidTransaction(e.to_string())
            })?;

        let header = self
            .get_block_header()
            .expect("block header is set in begin block"); //TODO: return error
        let height = header.height;

        let consensus_params = {
            let multi_store = &mut *self.multi_store.write().expect(POISONED_LOCK);
            let ctx = SimpleContext::new(multi_store.into(), height);
            self.baseapp_params_keeper.consensus_params(&ctx)
        };

        let mut ctx = mode.build_ctx(
            height,
            header,
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

        let mut multi_store = self.multi_store.write().expect(POISONED_LOCK);
        MD::commit(ctx, &mut *multi_store);

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
