mod handler;
pub mod param;
pub mod text;
pub mod upgrade;

use ::upgrade::{keeper::UpgradeKeeper, UpgradeHandler};
use gears::{
    application::keepers::params::ParamsKeeper, core::errors::CoreError, derive::AppMessage,
    params::ParamsSubspaceKey, store::StoreKey,
};
pub use handler::*;
use ibc_proto::google::protobuf::Any;
use param::{ParamChangeProposalHandler, ParameterChangeProposal};
use serde::{Deserialize, Serialize};
use text::TextProposal;
use upgrade::{CancelSoftwareUpgradeProposal, SoftwareUpgradeProposal, UpgradeProposalHandler};

pub trait Proposal:
    Clone
    + std::fmt::Debug
    + Send
    + Sync
    + serde::Serialize
    + TryFrom<Any, Error = CoreError>
    + Into<Any>
    + 'static
{
}

#[derive(Debug, Clone, AppMessage, Deserialize)]
pub enum Proposals<PSK: ParamsSubspaceKey> {
    #[msg(url(path = TextProposal::TYPE_URL))]
    Text(TextProposal),
    #[msg(url(path = ParameterChangeProposal::<PSK>::TYPE_URL))]
    Params(ParameterChangeProposal<PSK>),
    #[msg(url(path = SoftwareUpgradeProposal::TYPE_URL))]
    Upgrade(SoftwareUpgradeProposal),
    #[msg(url(path = CancelSoftwareUpgradeProposal::TYPE_URL))]
    CancelUpgrade(CancelSoftwareUpgradeProposal),
}

impl<PSK: ParamsSubspaceKey> Serialize for Proposals<PSK> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Proposals::Text(inner) => inner.serialize(serializer),
            Proposals::Params(inner) => inner.serialize(serializer),
            Proposals::Upgrade(inner) => inner.serialize(serializer),
            Proposals::CancelUpgrade(inner) => inner.serialize(serializer),
        }
    }
}

impl<PSK: ParamsSubspaceKey> Proposal for Proposals<PSK> {}

#[derive(Debug, Clone)]
pub struct ProposalsHandler<SK, PSK, PK, M, UH> {
    params_handler: ParamChangeProposalHandler<PK, SK, PSK>,
    upgrade_handler: UpgradeProposalHandler<SK, M, UH>,
}

impl<SK, PSK, PK, M, UH> ProposalsHandler<SK, PSK, PK, M, UH> {
    pub fn new(keeper: UpgradeKeeper<SK, M, UH>) -> Self {
        Self {
            params_handler: ParamChangeProposalHandler::new(),
            upgrade_handler: UpgradeProposalHandler::new(keeper),
        }
    }
}

impl<
        PSK: ParamsSubspaceKey,
        SK: StoreKey,
        PK: ParamsKeeper<PSK>,
        M: ::upgrade::Module + TryFrom<Vec<u8>, Error = anyhow::Error>,
        UH: UpgradeHandler,
    > ProposalHandler<Proposals<PSK>, SK> for ProposalsHandler<SK, PSK, PK, M, UH>
{
    fn handle<
        CTX: gears::context::InfallibleContextMut<DB, SK>,
        DB: gears::store::database::Database,
    >(
        &self,
        proposal: Proposals<PSK>,
        ctx: &mut CTX,
    ) -> Result<(), ProposalHandlingError> {
        match proposal {
            Proposals::Text(_) => Ok(()),
            Proposals::Params(proposal) => self.params_handler.handle(proposal, ctx),
            Proposals::Upgrade(proposal) => self.upgrade_handler.handle(proposal, ctx),
            Proposals::CancelUpgrade(proposal) => self.upgrade_handler.handle(proposal, ctx),
        }
    }

    fn check(proposal: &Proposals<PSK>) -> bool {
        match proposal {
            Proposals::Params(proposal) => {
                ParamChangeProposalHandler::<PK, SK, PSK>::check(proposal)
            }
            _ => true,
        }
    }
}
