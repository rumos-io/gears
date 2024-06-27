use std::collections::HashSet;

use crate::{
    application::keepers::params::ParamsKeeper,
    params::{ParamsSerialize, ParamsSubspaceKey},
    x::submission::{param::ParameterChangeProposal, text::TextProposal},
};

use super::{SubmissionCheckHandler, SubmissionHandler};

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
