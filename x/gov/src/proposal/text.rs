use std::marker::PhantomData;

use gears::{
    context::InfallibleContextMut,
    derive::{AppMessage, Protobuf},
    store::StoreKey,
};
use serde::{Deserialize, Serialize};

use super::handler::{ProposalHandler, SubmissionHandlingError};

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
pub struct TextSubmissionHandler<SK>(PhantomData<SK>);

impl<SK: StoreKey> ProposalHandler<TextProposal, SK> for TextSubmissionHandler<SK> {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: gears::store::database::Database>(
        &self,
        _proposal: TextProposal,
        _ctx: &mut CTX,
    ) -> Result<(), SubmissionHandlingError> {
        Ok(())
    }

    fn check(_proposal: &TextProposal) -> bool {
        true
    }
}
