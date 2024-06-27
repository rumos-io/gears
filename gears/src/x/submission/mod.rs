use std::{collections::HashSet, marker::PhantomData};

use database::Database;
use error::SubmissionError;
use kv_store::StoreKey;
use param::{ParamChange, ParameterChangeProposal};
use text::TextProposal;

use crate::{
    application::keepers::params::ParamsKeeper,
    context::{InfallibleContextMut, TransactionalContext},
    params::{gas::subspace_mut, infallible_subspace_mut, ParamsSerialize, ParamsSubspaceKey},
};

pub mod error;
pub mod param;
pub mod text;

#[derive(Debug, Default)]
pub struct TODOSubmissionHandler<PSK, P>(PhantomData<PSK>, PhantomData<P>);

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

impl<PSK: ParamsSubspaceKey, T: SubmissionHandler<PSK, ParameterChangeProposal<PSK>>>
    SubmissionCheckHandler<PSK, ParameterChangeProposal<PSK>> for T
{
    fn submission_check<PK: ParamsKeeper<PSK>>(
        &self,
        proposal: &ParameterChangeProposal<PSK>,
    ) -> bool {
        let set = <PK::Param as ParamsSerialize>::keys()
            .keys()
            .map(|this| this.as_bytes())
            .collect::<HashSet<_>>();

        proposal
            .changes
            .iter()
            .map(|this| &this.key)
            .all(|this| set.contains(this.as_slice()))
    }
}

impl<PSK: ParamsSubspaceKey, T: SubmissionHandler<PSK, TextProposal>>
    SubmissionCheckHandler<PSK, TextProposal> for T
{
    fn submission_check<PK: ParamsKeeper<PSK>>(&self, _proposal: &TextProposal) -> bool {
        true
    }
}

impl<PSK: ParamsSubspaceKey> SubmissionHandler<PSK, ParameterChangeProposal<PSK>>
    for TODOSubmissionHandler<PSK, ParameterChangeProposal<PSK>>
{
    fn handle<
        CTX: TransactionalContext<DB, SK>,
        PK: ParamsKeeper<PSK>,
        DB: Database,
        SK: StoreKey,
    >(
        &self,
        proposal: ParameterChangeProposal<PSK>,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> Result<(), SubmissionError> {
        if !self.submission_check::<PK>(&proposal) {
            Err(anyhow::anyhow!(
                "Can't handle this proposal: no such keys in subspace"
            ))?
        }

        let mut store = subspace_mut(ctx, keeper.psk());

        for ParamChange {
            subspace: _,
            key,
            value,
        } in proposal.changes
        {
            store.raw_key_set(key, value)?;
        }

        Ok(())
    }

    fn infallible_gas_handle<
        CTX: InfallibleContextMut<DB, SK>,
        PK: ParamsKeeper<PSK>,
        DB: Database,
        SK: StoreKey,
    >(
        &self,
        proposal: ParameterChangeProposal<PSK>,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()> {
        if !self.submission_check::<PK>(&proposal) {
            Err(anyhow::anyhow!(
                "Can't handle this proposal: no such keys in subspace"
            ))?
        }

        let mut store = infallible_subspace_mut(ctx, keeper.psk());

        for ParamChange {
            subspace: _,
            key,
            value,
        } in proposal.changes
        {
            store.raw_key_set(key, value);
        }

        Ok(())
    }
}
