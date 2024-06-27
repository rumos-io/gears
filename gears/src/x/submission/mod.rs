use std::collections::HashSet;

use param::ParameterChangeProposal;
use text::TextProposal;

use crate::{
    application::keepers::params::ParamsKeeper,
    context::TransactionalContext,
    params::{ParamsSerialize, ParamsSubspaceKey},
};

pub mod param;
pub mod text;

pub trait SubmissionHandler<PSK: ParamsSubspaceKey, P> {
    fn handle<CTX: TransactionalContext<DB, SK>, PK: ParamsKeeper<PSK>, DB, SK>(
        &self,
        proposal: P,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()>;
}

pub trait SubmissionCheckHandler<PSK: ParamsSubspaceKey, P>: SubmissionHandler<PSK, P> {
    fn submission_check<PK: ParamsKeeper<PSK>>(&self, proposal: &P) -> bool;
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
