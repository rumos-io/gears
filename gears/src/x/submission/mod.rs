use crate::{
    application::keepers::params::ParamsKeeper, context::TransactionalContext,
    params::ParamsSubspaceKey,
};

use core_types::{any::google::Any, errors::CoreError};

pub mod param;
pub mod text;

pub trait SubmissionHandler<PSK: ParamsSubspaceKey> {
    type Submission: TryFrom<Any, Error = CoreError>;

    fn handle<CTX: TransactionalContext<DB, SK>, PK: ParamsKeeper<PSK>, DB, SK>(
        &self,
        proposal: Self::Submission,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()>;
}
