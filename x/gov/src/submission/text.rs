use std::marker::PhantomData;

use gears::{
    application::keepers::params::ParamsKeeper,
    context::InfallibleContextMut,
    derive::{AppMessage, Protobuf},
    params::ParamsSubspaceKey,
};
use serde::{Deserialize, Serialize};

use super::handler::{SubmissionHandler, SubmissionHandlingError};

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::TextProposal;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Protobuf, AppMessage)]
#[proto(raw = "inner::TextProposal")]
#[msg(url = "/cosmos.params.v1beta1/TextProposal")]
pub struct TextProposal {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Default)]
pub struct TextSubmissionHandler<PK>(PhantomData<PK>);

impl<PSK: ParamsSubspaceKey, PK: ParamsKeeper<PSK>> SubmissionHandler<PK, PSK, TextProposal>
    for TextSubmissionHandler<PK>
{
    fn handle<
        CTX: InfallibleContextMut<DB, SK>,
        DB: gears::store::database::Database,
        SK: gears::store::StoreKey,
    >(
        _proposal: TextProposal,
        _ctx: &mut CTX,
        _keeper: &PSK,
    ) -> Result<(), SubmissionHandlingError> {
        Ok(())
    }
}
