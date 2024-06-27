use std::{collections::HashSet, marker::PhantomData};

use anyhow::anyhow;
use gears::{
    application::keepers::params::ParamsKeeper,
    context::TransactionalContext,
    params::{ParamsSerialize, ParamsSubspaceKey},
    x::submission::{param::ParameterChangeProposal, SubmissionCheckHandler, SubmissionHandler},
};

#[derive(Debug, Default)]
pub struct AuthSubmissionHandler<PSK>(PhantomData<PSK>);

impl<PSK: ParamsSubspaceKey> SubmissionHandler<PSK, ParameterChangeProposal<PSK>>
    for AuthSubmissionHandler<PSK>
{
    fn handle<CTX: TransactionalContext<DB, SK>, PK: ParamsKeeper<PSK>, DB, SK>(
        &self,
        proposal: ParameterChangeProposal<PSK>,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()> {
        if !self.submission_check::<PK>(&proposal) {
            Err(anyhow!(
                "Can't handle this proposal: no such keys in subspace"
            ))?
        }

        todo!()
    }
}
