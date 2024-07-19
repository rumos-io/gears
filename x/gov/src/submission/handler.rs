use std::marker::PhantomData;

use gears::context::TransactionalContext;
use gears::params::gas::subspace_mut;
use gears::store::database::Database;
use gears::store::StoreKey;

use gears::types::store::gas::errors::GasStoreErrors;
use gears::{
    application::keepers::params::ParamsKeeper, context::InfallibleContextMut,
    params::ParamsSubspaceKey,
};

use super::param::ParamChange;

pub trait SubmissionHandler<PK: ParamsKeeper<PSK>, PSK: ParamsSubspaceKey, P> {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
        proposal: P,
        ctx: &mut CTX,
        subspace: &PSK,
    ) -> Result<(), SubmissionHandlingError>;
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SubmissionHandlingError {
    #[error("Can't handle this proposal: no such keys in subspace")]
    KeyNotFound,
    #[error("Can't handle this proposal: invalid bytes")]
    InvalidProposal,
    #[error("Can't handle this proposal: {0}")]
    Gas(#[from] GasStoreErrors),
}

#[derive(Debug)]
pub struct ParamChangeSubmissionHandler<PK>(PhantomData<PK>);

impl<PSK: ParamsSubspaceKey, PK: ParamsKeeper<PSK>> SubmissionHandler<PK, PSK, ParamChange<PSK>>
    for ParamChangeSubmissionHandler<PK>
{
    fn handle<CTX: TransactionalContext<DB, SK>, DB: Database, SK: StoreKey>(
        proposal: ParamChange<PSK>,
        ctx: &mut CTX,
        subspace_key: &PSK,
    ) -> Result<(), SubmissionHandlingError> {
        if !PK::check_key(&proposal.key) {
            Err(SubmissionHandlingError::KeyNotFound)?
        }

        if !PK::validate(&proposal.key, &proposal.value) {
            Err(SubmissionHandlingError::InvalidProposal)?
        }

        let mut store = subspace_mut(ctx, subspace_key);

        store.raw_key_set(proposal.key, proposal.value)?;

        Ok(())
    }
}
