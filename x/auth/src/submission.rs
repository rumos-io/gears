use std::marker::PhantomData;

use gears::{
    application::keepers::params::ParamsKeeper,
    context::TransactionalContext,
    params::ParamsSubspaceKey,
    x::submission::{param::ParameterChangeProposal, SubmissionHandler},
};

#[derive(Debug, Default)]
pub struct AuthSubmissionHandler<PSK>(PhantomData<PSK>);

impl<PSK: ParamsSubspaceKey> SubmissionHandler<PSK> for AuthSubmissionHandler<PSK> {
    type Submission = ParameterChangeProposal<PSK>;

    fn handle<CTX: TransactionalContext<DB, SK>, PK: ParamsKeeper<PSK>, DB, SK>(
        &self,
        proposal: Self::Submission,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
