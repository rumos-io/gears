use deposit::Deposit;
use gears::{
    derive::AppMessage,
    signing::{
        handler::MetadataGetter,
        renderer::value_renderer::{RenderError, ValueRenderer},
    },
    types::rendering::screen::Screen,
};
use proposal::MsgSubmitProposal;
use serde::Serialize;
use vote::Vote;
use weighted_vote::MsgVoteWeighted;

pub mod deposit;
pub mod proposal;
pub mod vote;
pub mod weighted_vote;

#[derive(Debug, Clone, Serialize, AppMessage)]
pub enum GovMsg {
    #[msg(url(path = Deposit::TYPE_URL))]
    Deposit(Deposit),
    #[msg(url(path = Vote::TYPE_URL))]
    Vote(Vote),
    #[msg(url(path = MsgVoteWeighted::TYPE_URL))]
    Weighted(MsgVoteWeighted),
    #[msg(url(path = MsgSubmitProposal::TYPE_URL))]
    Proposal(MsgSubmitProposal),
}

impl ValueRenderer for GovMsg {
    fn format<MG: MetadataGetter>(&self, _: &MG) -> Result<Vec<Screen>, RenderError> {
        Err(RenderError::NotImplemented)
    }
}
