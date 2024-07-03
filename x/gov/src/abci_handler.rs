use bytes::Bytes;
use gears::{
    application::handlers::node::ABCIHandler,
    context::{
        block::BlockContext, init::InitContext, query::QueryContext, tx::TxContext,
        TransactionalContext,
    },
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::{
            event::{Event, EventAttribute},
            validator::ValidatorUpdate,
        },
        request::{end_block::RequestEndBlock, query::RequestQuery},
    },
    types::tx::raw::TxWithRaw,
    x::{
        keepers::{gov::GovernanceBankKeeper, staking::GovStakingKeeper},
        module::Module,
    },
};

use crate::{
    genesis::GovGenesisState,
    keeper::GovKeeper,
    msg::{deposit::Deposit, GovMsg},
    query::{GovQuery, GovQueryResponse},
    types::proposal::Proposal,
    ProposalHandler,
};

#[derive(Debug, Clone)]
pub struct GovAbciHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Module,
    BK: GovernanceBankKeeper<SK, M>,
    STK: GovStakingKeeper<SK, M>,
    PH: ProposalHandler<PSK, Proposal>,
> {
    keeper: GovKeeper<SK, PSK, M, BK, STK, PH>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: Module,
        BK: GovernanceBankKeeper<SK, M>,
        STK: GovStakingKeeper<SK, M>,
        PH: ProposalHandler<PSK, Proposal>,
    > GovAbciHandler<SK, PSK, M, BK, STK, PH>
{
    pub fn new(keeper: GovKeeper<SK, PSK, M, BK, STK, PH>) -> Self {
        Self { keeper }
    }
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: Module,
        BK: GovernanceBankKeeper<SK, M>,
        STK: GovStakingKeeper<SK, M>,
        PH: ProposalHandler<PSK, Proposal> + Clone + Send + Sync + 'static,
    > ABCIHandler for GovAbciHandler<SK, PSK, M, BK, STK, PH>
{
    type Message = GovMsg;

    type Genesis = GovGenesisState;

    type StoreKey = SK;

    type QReq = GovQuery;

    type QRes = GovQueryResponse;

    fn typed_query<DB: Database>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        todo!()
    }

    fn run_ante_checks<DB: Database>(
        &self,
        _ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        _tx: &TxWithRaw<Self::Message>,
    ) -> Result<(), AppError> {
        Ok(())
    }

    fn tx<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        msg: &Self::Message,
    ) -> Result<(), AppError> {
        enum EmitEvent {
            Regular,
            Deposit(u64),
            Proposal((String, Option<u64>)),
        }

        let (address_str, proposal) = match msg {
            GovMsg::Deposit(msg) => {
                let is_voting_started = self
                    .keeper
                    .deposit_add(ctx, msg.clone())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                match is_voting_started {
                    true => (
                        msg.depositor.to_string(),
                        EmitEvent::Deposit(msg.proposal_id),
                    ),
                    false => (msg.depositor.to_string(), EmitEvent::Regular),
                }
            }
            GovMsg::Vote(msg) => {
                self.keeper
                    .vote_add(ctx, msg.clone().into())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                (msg.voter.to_string(), EmitEvent::Regular)
            }
            GovMsg::Weighted(msg) => {
                self.keeper
                    .vote_add(ctx, msg.clone())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                (msg.voter.to_string(), EmitEvent::Regular)
            }
            GovMsg::Proposal(msg) => {
                let proposal_id = self
                    .keeper
                    .submit_proposal(ctx, msg.clone())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                let is_voting_started = self
                    .keeper
                    .deposit_add(
                        ctx,
                        Deposit {
                            proposal_id,
                            depositor: msg.proposer.clone(),
                            amount: msg.initial_deposit.clone(),
                        },
                    )
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                match is_voting_started {
                    true => (
                        msg.proposer.to_string(),
                        EmitEvent::Proposal((msg.content.type_url.clone(), Some(proposal_id))),
                    ),
                    false => (
                        msg.proposer.to_string(),
                        EmitEvent::Proposal((msg.content.type_url.clone(), None)),
                    ),
                }
            }
        };

        ctx.push_event(Event::new(
            "message",
            vec![
                EventAttribute::new("module".into(), "governance".into(), false),
                EventAttribute::new("sender".into(), address_str.into(), false),
            ],
        ));

        match proposal {
            EmitEvent::Regular => (),
            EmitEvent::Deposit(proposal) => {
                ctx.push_event(Event::new(
                    "proposal_deposit",
                    vec![EventAttribute::new(
                        "voting_period_start".into(),
                        proposal.to_string().into(),
                        false,
                    )],
                ));
            }
            EmitEvent::Proposal((proposal_type, proposal)) => {
                ctx.push_event(Event::new(
                    "submit_proposal",
                    match proposal {
                        Some(proposal_id) => vec![
                            EventAttribute::new(
                                "proposal_type".into(),
                                proposal_type.into(),
                                false,
                            ),
                            EventAttribute::new(
                                "voting_period_start".into(),
                                proposal_id.to_string().into(),
                                false,
                            ),
                        ],
                        None => vec![EventAttribute::new(
                            "proposal_type".into(),
                            proposal_type.into(),
                            false,
                        )],
                    },
                ));
            }
        }

        Ok(())
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    ) {
        self.keeper.init_genesis(ctx, genesis)
    }

    fn query<DB: Database>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: RequestQuery,
    ) -> Result<Bytes, AppError> {
        // match query.path.as_str() {

        // }

        todo!()
    }

    fn end_block<'a, DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, Self::StoreKey>,
        _request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        let events = self.keeper.end_block(ctx);

        ctx.append_events(events);

        Vec::new()
    }
}
