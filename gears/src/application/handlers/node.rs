use std::num::NonZero;

use crate::{
    baseapp::{errors::QueryError, genesis::Genesis, QueryRequest, QueryResponse},
    context::{block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext},
    types::tx::{raw::TxWithRaw, TxMessage},
};
use database::Database;
use kv_store::StoreKey;
use tendermint::types::{
    proto::validator::ValidatorUpdate,
    request::{begin_block::RequestBeginBlock, end_block::RequestEndBlock, query::RequestQuery},
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("error code must be greater than 0")]
pub struct ErrorCodeError;

#[derive(Debug, Clone)]
pub struct ErrorCode(NonZero<u16>);

impl ErrorCode {
    pub const fn new(code: NonZero<u16>) -> Self {
        Self(code)
    }

    pub const fn try_new(code: u16) -> Result<Self, ErrorCodeError> {
        match NonZero::new(code) {
            Some(var) => Ok(Self(var)),
            None => Err(ErrorCodeError),
        }
    }

    pub fn value(&self) -> u16 {
        self.0.get()
    }
}

pub trait ModuleInfo {
    const NAME: &'static str;
}

#[derive(Error, Debug, Clone)]
#[error("{msg}")]
pub struct TxError {
    pub msg: String,
    pub code: ErrorCode,
    pub codespace: &'static str,
}

pub trait ABCIHandler: Clone + Send + Sync + 'static {
    type Message: TxMessage;
    type Genesis: Genesis;
    type StoreKey: StoreKey;

    type QReq: QueryRequest;
    type QRes: QueryResponse;

    fn typed_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: Self::QReq,
    ) -> Self::QRes;

    fn run_ante_checks<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        tx: &TxWithRaw<Self::Message>,
    ) -> Result<(), TxError>;

    fn msg<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        msg: &Self::Message,
    ) -> Result<(), TxError>;

    #[allow(unused_variables)]
    fn begin_block<'a, DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, Self::StoreKey>,
        request: RequestBeginBlock,
    ) {
    }

    #[allow(unused_variables)]
    fn end_block<'a, DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, Self::StoreKey>,
        request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        Vec::new()
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    );

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: RequestQuery,
    ) -> Result<bytes::Bytes, QueryError>;
}
