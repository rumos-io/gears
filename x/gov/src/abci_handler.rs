use bytes::Bytes;
use gears::{
    application::handlers::node::ABCIHandler,
    context::{init::InitContext, query::QueryContext, tx::TxContext, TransactionalContext},
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::event::{Event, EventAttribute},
        request::query::RequestQuery,
    },
    types::tx::raw::TxWithRaw,
    x::{keepers::bank::BankKeeper, module::Module},
};

use crate::{
    genesis::GovGenesisState,
    keeper::GovKeeper,
    msg::GovMsg,
    query::{GovQueryRequest, GovQueryResponse},
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
        let (address_str, proposal) = match msg {
            GovMsg::Deposit(msg) => {
                let is_voting_started = self
                    .keeper
                    .deposit_add(ctx, msg.clone())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                match is_voting_started {
                    true => (msg.depositor.to_string(), Some(msg.proposal_id)),
                    false => (msg.depositor.to_string(), None),
                }
            }
            GovMsg::Vote(msg) => {
                self.keeper
                    .vote_add(ctx, msg.clone().into())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                (msg.voter.to_string(), None)
            }
            GovMsg::Weighted(msg) => {
                self.keeper
                    .vote_add(ctx, msg.clone())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                (msg.voter.to_string(), None)
            }
        };

        ctx.push_event(Event::new(
            "message",
            vec![
                EventAttribute::new("module".into(), "governance".into(), false),
                EventAttribute::new("sender".into(), address_str.into(), false),
            ],
        ));

        if let Some(proposal) = proposal {
            ctx.push_event(Event::new(
                "proposal_deposit",
                vec![EventAttribute::new(
                    "voting_period_start".into(),
                    proposal.to_string().into(),
                    false,
                )],
            ));
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
}
