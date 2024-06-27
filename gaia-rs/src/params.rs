use auth::AuthParamsKeeper;
use bank::BankParamsKeeper;
use gears::{
    application::keepers::params::ParamsKeeper,
    context::InfallibleContextMut,
    params::{ParamsDeserialize, ParamsSerialize},
    store::{database::Database, StoreKey},
    x::submission::{
        handler::{params::ParamChangeSubmissionHandler, SubmissionHandler},
        param::ParameterChangeProposal,
        text::{TextProposal, TextSubmissionHandler},
    },
};
use gov::{types::proposal::Proposal, ProposalHandler};

use crate::store_keys::GaiaParamsStoreKey;

#[derive(Debug)]
pub struct GaiaGovernanceHandler;

impl ProposalHandler<GaiaParamsStoreKey, Proposal> for GaiaGovernanceHandler {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
        &self,
        proposal: Proposal,
        ctx: &mut CTX,
    ) -> anyhow::Result<()> {
        match proposal.content.type_url.as_str() {
            ParameterChangeProposal::<GaiaParamsStoreKey>::TYPE_URL => {
                let msg: ParameterChangeProposal<GaiaParamsStoreKey> =
                    ParameterChangeProposal::try_from(proposal.content)?;

                for change in msg.changes {
                    match change.subspace.clone() {
                        space @ GaiaParamsStoreKey::Bank => ParamChangeSubmissionHandler::<
                            BankParamsKeeper<GaiaParamsStoreKey>,
                        >::handle(
                            change, ctx, &space
                        ),
                        space @ GaiaParamsStoreKey::Auth => ParamChangeSubmissionHandler::<
                            AuthParamsKeeper<GaiaParamsStoreKey>,
                        >::handle(
                            change, ctx, &space
                        ),
                        GaiaParamsStoreKey::BaseApp => {
                            Err(anyhow::anyhow!("not supported subspace"))
                        }
                        GaiaParamsStoreKey::Staking => {
                            Err(anyhow::anyhow!("not supported subspace"))
                        }
                        GaiaParamsStoreKey::IBC => Err(anyhow::anyhow!("not supported subspace")),
                        GaiaParamsStoreKey::Capability => {
                            Err(anyhow::anyhow!("not supported subspace"))
                        }
                    }?;
                }

                Ok(())
            }
            TextProposal::TYPE_URL => TextSubmissionHandler::<DummyParamsKeeper>::handle(
                proposal.content.try_into()?,
                ctx,
                &DUMMY_PARAMS,
            ),
            _ => Err(anyhow::anyhow!("Invalid proposal content")),
        }
    }
}

const DUMMY_PARAMS: GaiaParamsStoreKey = GaiaParamsStoreKey::Auth;

/// We need dummy keeper for textual propose which doesn't change any value, but need to satisfy api
#[derive(Debug, Default, Clone)]
struct DummyParamsKeeper;

impl ParamsKeeper<GaiaParamsStoreKey> for DummyParamsKeeper {
    type Param = DummyParams;

    fn psk(&self) -> &GaiaParamsStoreKey {
        &DUMMY_PARAMS
    }
}

#[derive(Debug, Default, Clone)]
struct DummyParams;

impl ParamsSerialize for DummyParams {
    fn keys() -> std::collections::HashMap<&'static str, gears::params::ParamKind> {
        Default::default()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        Default::default()
    }
}

impl ParamsDeserialize for DummyParams {
    fn from_raw(_: std::collections::HashMap<&'static str, Vec<u8>>) -> Self {
        Self
    }
}
