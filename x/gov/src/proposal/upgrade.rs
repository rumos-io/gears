use gears::{
    context::InfallibleContextMut,
    derive::{AppMessage, Protobuf},
    store::StoreKey,
};
use serde::{Deserialize, Serialize};
use upgrade::{keeper::UpgradeKeeper, types::plan::Plan, Module, UpgradeHandler};

use super::handler::{ProposalHandler, SubmissionHandlingError};

mod inner {

    // pub use ibc_proto::cosmos::upgrade::v1beta1::MsgCancelUpgrade;
    // pub use ibc_proto::cosmos::upgrade::v1beta1::MsgSoftwareUpgrade;
    // Deprecated, but we need to use it
    pub use ibc_proto::cosmos::upgrade::v1beta1::CancelSoftwareUpgradeProposal;
    pub use ibc_proto::cosmos::upgrade::v1beta1::SoftwareUpgradeProposal;
}

// #[derive(Debug, Clone, Protobuf, Serialize, Deserialize, AppMessage)]
// #[proto(raw = "inner::MsgCancelUpgrade")]
// #[serde(try_from = "inner::MsgCancelUpgrade", into = "inner::MsgCancelUpgrade")]
// #[msg(url = "/cosmos.upgrade.v1beta1/MsgCancelUpgrade")]
// pub struct MsgCancelUpgrade {
//     pub authority: AccAddress,
// }

// #[derive(Debug, Clone, Protobuf, Serialize, Deserialize, AppMessage)]
// #[proto(raw = "inner::MsgSoftwareUpgrade")]
// #[serde(
//     try_from = "inner::MsgSoftwareUpgrade",
//     into = "inner::MsgSoftwareUpgrade"
// )]
// #[msg(url = "/cosmos.upgrade.v1beta1/MsgSoftwareUpgrade")]
// pub struct MsgSoftwareUpgrade {
//     pub authority: AccAddress,
//     #[proto(optional)]
//     pub plan: Plan,
// }

#[derive(Debug, Clone, Protobuf, Serialize, Deserialize, AppMessage)]
#[proto(raw = "inner::CancelSoftwareUpgradeProposal")]
#[serde(
    try_from = "inner::CancelSoftwareUpgradeProposal",
    into = "inner::CancelSoftwareUpgradeProposal"
)]
#[msg(url = "/cosmos.upgrade.v1beta1/CancelSoftwareUpgradeProposal")]
pub struct CancelSoftwareUpgradeProposal {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Protobuf, Serialize, Deserialize, AppMessage)]
#[proto(raw = "inner::SoftwareUpgradeProposal")]
#[serde(
    try_from = "inner::SoftwareUpgradeProposal",
    into = "inner::SoftwareUpgradeProposal"
)]
#[msg(url = "/cosmos.upgrade.v1beta1/SoftwareUpgradeProposal")]
pub struct SoftwareUpgradeProposal {
    pub title: String,
    pub description: String,
    #[proto(optional)]
    pub plan: Plan,
}

#[derive(Debug)]
pub struct UpgradeSubmissionHandler<SK, M, UH> {
    keeper: UpgradeKeeper<SK, M, UH>,
}

impl<SK, M, UH> UpgradeSubmissionHandler<SK, M, UH> {
    pub fn new(keeper: UpgradeKeeper<SK, M, UH>) -> Self {
        Self { keeper }
    }
}

impl<SK: StoreKey, M: Module, UH: UpgradeHandler> ProposalHandler<SoftwareUpgradeProposal, SK>
    for UpgradeSubmissionHandler<SK, M, UH>
where
    <M as TryFrom<Vec<u8>>>::Error: std::fmt::Display + std::fmt::Debug,
{
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: gears::store::database::Database>(
        &self,
        SoftwareUpgradeProposal {
            plan,
            title: _,
            description: _,
        }: SoftwareUpgradeProposal,
        ctx: &mut CTX,
    ) -> Result<(), SubmissionHandlingError> {
        self.keeper
            .schedule_upgrade(ctx, plan, true)
            .map_err(|e| SubmissionHandlingError::Other(e.to_string()))?;

        Ok(())
    }

    fn check(_proposal: &SoftwareUpgradeProposal) -> bool {
        true
    }
}

impl<SK: StoreKey, M: Module, UH: UpgradeHandler>
    ProposalHandler<CancelSoftwareUpgradeProposal, SK> for UpgradeSubmissionHandler<SK, M, UH>
where
    <M as TryFrom<Vec<u8>>>::Error: std::fmt::Display + std::fmt::Debug,
{
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: gears::store::database::Database>(
        &self,
        CancelSoftwareUpgradeProposal {
            title: _,
            description: _,
        }: CancelSoftwareUpgradeProposal,
        ctx: &mut CTX,
    ) -> Result<(), SubmissionHandlingError> {
        self.keeper.delete_upgrade_plan(ctx);

        Ok(())
    }

    fn check(_proposal: &CancelSoftwareUpgradeProposal) -> bool {
        true
    }
}
