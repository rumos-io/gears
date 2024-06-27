use database::Database;
use kv_store::StoreKey;

use crate::{
    application::keepers::params::ParamsKeeper, context::InfallibleContextMut,
    params::ParamsSubspaceKey,
};

mod handler_impl;
pub mod params;

pub trait SubmissionCheckHandler<PSK: ParamsSubspaceKey, P>: SubmissionHandler<PSK, P> {
    fn submission_check<PK: ParamsKeeper<PSK>>(proposal: &P) -> bool;
}

pub trait SubmissionHandler<PSK: ParamsSubspaceKey, P> {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
        proposal: P,
        ctx: &mut CTX,
        keeper: &PSK,
    ) -> anyhow::Result<()>;
}
