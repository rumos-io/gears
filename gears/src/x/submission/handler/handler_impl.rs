use std::collections::HashSet;

use crate::{
    application::keepers::params::ParamsKeeper,
    params::{ParamsSerialize, ParamsSubspaceKey},
    x::submission::{param::ParamChange, text::TextProposal},
};

use super::{SubmissionCheckHandler, SubmissionHandler};

impl<PSK: ParamsSubspaceKey, T: SubmissionHandler<PSK, ParamChange<PSK>>>
    SubmissionCheckHandler<PSK, ParamChange<PSK>> for T
{
    fn submission_check<PK: ParamsKeeper<PSK>>(proposal: &ParamChange<PSK>) -> bool {
        <PK::Param as ParamsSerialize>::keys()
            .keys()
            .map(|this| this.as_bytes())
            .collect::<HashSet<_>>()
            .contains(proposal.key.as_slice())
    }
}

impl<PSK: ParamsSubspaceKey, T: SubmissionHandler<PSK, TextProposal>>
    SubmissionCheckHandler<PSK, TextProposal> for T
{
    fn submission_check<PK: ParamsKeeper<PSK>>(_proposal: &TextProposal) -> bool {
        true
    }
}
