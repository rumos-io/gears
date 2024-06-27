use std::{collections::HashSet, marker::PhantomData};

use gears::{
    application::keepers::params::ParamsKeeper,
    context::TransactionalContext,
    params::{ParamsSerialize, ParamsSubspaceKey},
    x::submission::{param::ParameterChangeProposal, SubmissionHandler},
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
        todo!()
    }
}
