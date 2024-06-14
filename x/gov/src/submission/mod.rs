// use gears::core::errors::CoreError;
// use ibc_proto::google::protobuf::Any;

use gears::{
    application::keepers::params::ParamsKeeper, context::TransactionalContext,
    params::ParamsSubspaceKey,
};

pub mod param;

// : TryFrom<Any, Error = CoreError>
pub trait ProposalSubmission {
    fn handle<
        CTX: TransactionalContext<DB, SK>,
        PK: ParamsKeeper<PSK>,
        PSK: ParamsSubspaceKey,
        DB,
        SK,
    >(
        &self,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()>;
}
