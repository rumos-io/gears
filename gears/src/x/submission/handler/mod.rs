use database::Database;
use kv_store::StoreKey;

use crate::{
    application::keepers::params::ParamsKeeper,
    context::{InfallibleContextMut, TransactionalContext},
    params::ParamsSubspaceKey,
};

use super::error::SubmissionError;

mod handler_impl;
pub mod params;

pub trait SubmissionCheckHandler<PSK: ParamsSubspaceKey, P>: SubmissionHandler<PSK, P> {
    fn submission_check<PK: ParamsKeeper<PSK>>(&self, proposal: &P) -> bool;
}

pub trait SubmissionHandler<PSK: ParamsSubspaceKey, P> {
    fn handle<
        CTX: TransactionalContext<DB, SK>,
        PK: ParamsKeeper<PSK>,
        DB: Database,
        SK: StoreKey,
    >(
        &self,
        proposal: P,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> Result<(), SubmissionError>;

    fn infallible_gas_handle<
        CTX: InfallibleContextMut<DB, SK>,
        PK: ParamsKeeper<PSK>,
        DB: Database,
        SK: StoreKey,
    >(
        &self,
        proposal: P,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()>;
}
