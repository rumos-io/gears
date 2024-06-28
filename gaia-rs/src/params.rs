use auth::AuthParamsKeeper;
use bank::BankParamsKeeper;
use gears::{
    application::keepers::params::ParamsKeeper,
    baseapp::BaseAppParamsKeeper,
    context::InfallibleContextMut,
    params::{ParamsDeserialize, ParamsSerialize},
    store::{database::Database, StoreKey},
};
use gov::{
    submission::{
        handler::{
            params::ParamChangeSubmissionHandler, SubmissionCheckHandler, SubmissionHandler,
        },
        param::ParameterChangeProposal,
        text::{TextProposal, TextSubmissionHandler},
    },
    types::proposal::Proposal,
    ProposalHandler,
};
use staking::StakingParamsKeeper;

use crate::store_keys::GaiaParamsStoreKey;

#[derive(Debug)]
pub struct GaiaProposalHandler;

impl ProposalHandler<GaiaParamsStoreKey, Proposal> for GaiaProposalHandler {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
        &self,
        proposal: &Proposal,
        ctx: &mut CTX,
    ) -> anyhow::Result<()> {
        match proposal.content.type_url.as_str() {
            ParameterChangeProposal::<GaiaParamsStoreKey>::TYPE_URL => {
                let msg: ParameterChangeProposal<GaiaParamsStoreKey> =
                    ParameterChangeProposal::try_from(proposal.content.clone())?;

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
                        space @ GaiaParamsStoreKey::BaseApp => ParamChangeSubmissionHandler::<
                            BaseAppParamsKeeper<GaiaParamsStoreKey>,
                        >::handle(
                            change, ctx, &space
                        ),
                        space @ GaiaParamsStoreKey::Staking => ParamChangeSubmissionHandler::<
                            StakingParamsKeeper<GaiaParamsStoreKey>,
                        >::handle(
                            change, ctx, &space
                        ),
                        GaiaParamsStoreKey::IBC => Err(anyhow::anyhow!("not supported subspace")),
                        GaiaParamsStoreKey::Capability => {
                            Err(anyhow::anyhow!("not supported subspace"))
                        }
                    }?;
                }

                Ok(())
            }
            TextProposal::TYPE_URL => TextSubmissionHandler::<DummyParamsKeeper>::handle(
                proposal.content.clone().try_into()?,
                ctx,
                &DUMMY_PARAMS,
            ),
            _ => Err(anyhow::anyhow!("Invalid proposal content")),
        }
    }

    fn check(proposal: &Proposal) -> bool {
        match proposal.content.type_url.as_str() {
            ParameterChangeProposal::<GaiaParamsStoreKey>::TYPE_URL => {
                let msg: Result<ParameterChangeProposal<_>, gears::core::errors::CoreError> =
                    ParameterChangeProposal::try_from(proposal.content.clone());

                match msg {
                    Ok(msg) => {
                        for change in msg.changes {
                            if !match change.subspace {
                                GaiaParamsStoreKey::Bank => {
                                    ParamChangeSubmissionHandler::<
                                        BankParamsKeeper<GaiaParamsStoreKey>,
                                    >::submission_check::<BankParamsKeeper<GaiaParamsStoreKey>>(
                                        &change,
                                    )
                                }
                                GaiaParamsStoreKey::Auth => {
                                    ParamChangeSubmissionHandler::<
                                        AuthParamsKeeper<GaiaParamsStoreKey>,
                                    >::submission_check::<AuthParamsKeeper<GaiaParamsStoreKey>>(
                                        &change,
                                    )
                                }
                                GaiaParamsStoreKey::BaseApp => {
                                    ParamChangeSubmissionHandler::<
                                        BaseAppParamsKeeper<GaiaParamsStoreKey>,
                                    >::submission_check::<BaseAppParamsKeeper<GaiaParamsStoreKey>>(
                                        &change,
                                    )
                                }
                                GaiaParamsStoreKey::Staking => {
                                    ParamChangeSubmissionHandler::<
                                        StakingParamsKeeper<GaiaParamsStoreKey>,
                                    >::submission_check::<StakingParamsKeeper<GaiaParamsStoreKey>>(
                                        &change,
                                    )
                                }
                                GaiaParamsStoreKey::IBC => false,
                                GaiaParamsStoreKey::Capability => false,
                            } {
                                return false;
                            }
                        }

                        true
                    }
                    Err(_) => false,
                }
            }
            TextProposal::TYPE_URL => true,
            _ => false,
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
    fn keys() -> std::collections::HashSet<&'static str> {
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
