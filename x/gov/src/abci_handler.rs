use bytes::Bytes;
use chrono::DateTime;
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
    types::{store::gas::ext::GasResultExt, tx::raw::TxWithRaw},
    x::{keepers::bank::BankKeeper, module::Module},
};

use crate::{
    genesis::GovGenesisState,
    keeper::GovKeeper,
    msg::{deposit::MsgDeposit, GovMsg},
    query::{GovQueryRequest, GovQueryResponse},
    types::proposal::{
        active_iter::ActiveProposalIterator, inactive_iter::InactiveProposalIterator,
    },
};

#[derive(Debug, Clone)]
pub struct GovAbciHandler<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>> {
    keeper: GovKeeper<SK, PSK, M, BK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>>
    GovAbciHandler<SK, PSK, M, BK>
{
    pub fn new(keeper: GovKeeper<SK, PSK, M, BK>) -> Self {
        Self { keeper }
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>> ABCIHandler
    for GovAbciHandler<SK, PSK, M, BK>
{
    type Message = GovMsg;

    type Genesis = GovGenesisState;

    type StoreKey = SK;

    type QReq = GovQueryRequest;

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
                        MsgDeposit {
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
        todo!()
    }

    fn end_block<'a, DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, Self::StoreKey>,
        _request: RequestEndBlock,
    ) -> Vec<ValidatorUpdate> {
        let time = DateTime::from_timestamp(ctx.header.time.seconds, ctx.header.time.nanos as u32)
            .unwrap(); // TODO
        let store = ctx.kv_store(&self.keeper.store_key).into();
        {
            let inactive_iter = InactiveProposalIterator::new(&store, &time);

            for var in inactive_iter {
                let ((_proposal_id, _date), _val) = var.unwrap_gas();
            }
        }

        {
            let active_iter = ActiveProposalIterator::new(&store, &time);

            for var in active_iter {
                let ((_proposal_id, _date), _val) = var.unwrap_gas();
            }
        }

        Vec::new()
    }
}
