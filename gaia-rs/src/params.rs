use bank::BankParamsKeeper;
use gears::{
    application::keepers::params::ParamsKeeper,
    context::{InfallibleContextMut, TransactionalContext},
    store::{database::Database, StoreKey},
    x::submission::{
        error::SubmissionError,
        handler::SubmissionHandler,
        param::ParameterChangeProposal,
        text::{TextProposal, TextSubmissionHandler},
    },
};
use gov::{msg::proposal::MsgSubmitProposal, types::proposal::Proposal};

use crate::store_keys::GaiaParamsStoreKey;

#[derive(Debug, Default)]
pub struct GaiaGovernanceHandler;

impl SubmissionHandler<GaiaParamsStoreKey, Proposal> for GaiaGovernanceHandler {
    fn handle<
        CTX: TransactionalContext<DB, SK>,
        PK: ParamsKeeper<GaiaParamsStoreKey>,
        DB: Database,
        SK: StoreKey,
    >(
        &self,
        proposal: Proposal,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> Result<(), SubmissionError> {
        todo!()
    }

    fn infallible_gas_handle<
        CTX: InfallibleContextMut<DB, SK>,
        PK: ParamsKeeper<GaiaParamsStoreKey>,
        DB: Database,
        SK: StoreKey,
    >(
        &self,
        proposal: Proposal,
        ctx: &mut CTX,
        keeper: &mut PK,
    ) -> anyhow::Result<()> {
        match proposal.content.type_url.as_str() {
            ParameterChangeProposal::<GaiaParamsStoreKey>::TYPE_URL => {
                let msg: ParameterChangeProposal<GaiaParamsStoreKey> =
                    ParameterChangeProposal::try_from(proposal.content)?;

                for change in msg.changes {
                    match change.subspace {
                        space @ GaiaParamsStoreKey::Bank => {
                            todo!()
                        }
                        space @ GaiaParamsStoreKey::Auth => todo!(),
                        space @ GaiaParamsStoreKey::BaseApp => todo!(),
                        space @ GaiaParamsStoreKey::Staking => todo!(),
                        space @ GaiaParamsStoreKey::IBC => todo!(),
                        space @ GaiaParamsStoreKey::Capability => todo!(),
                    }
                }

                Ok(())
            }
            TextProposal::TYPE_URL => TextSubmissionHandler::default().infallible_gas_handle(
                proposal.content.try_into()?,
                ctx,
                keeper,
            ),
            _ => Err(anyhow::anyhow!("Invalid proposal content")),
        }
    }
}
