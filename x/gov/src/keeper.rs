use std::{marker::PhantomData, ops::Add};

use anyhow::anyhow;
use chrono::DateTime;
use gears::{
    application::keepers::params::ParamsKeeper,
    context::{
        block::BlockContext, init::InitContext, tx::TxContext, QueryableContext,
        TransactionalContext,
    },
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::proto::event::{Event, EventAttribute},
    types::{
        address::AccAddress,
        store::gas::{errors::GasStoreErrors, ext::GasResultExt},
    },
    x::{keepers::bank::BankKeeper, module::Module},
};

use crate::{
    errors::SERDE_JSON_CONVERSION,
    genesis::GovGenesisState,
    msg::{deposit::MsgDeposit, proposal::MsgSubmitProposal, weighted_vote::MsgVoteWeighted},
    params::GovParamsKeeper,
    types::proposal::{
        active_iter::ActiveProposalIterator, inactive_iter::InactiveProposalIterator, Proposal,
        ProposalStatus,
    },
};

const PROPOSAL_ID_KEY: [u8; 1] = [0x03];
pub(crate) const KEY_PROPOSAL_PREFIX: [u8; 1] = [0x00];

#[derive(Debug, Clone)]
pub struct GovKeeper<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>> {
    store_key: SK,
    gov_params_keeper: GovParamsKeeper<PSK>,
    gov_mod: M,
    bank_keeper: BK,
    _bank_marker: PhantomData<M>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>>
    GovKeeper<SK, PSK, M, BK>
{
    pub fn new(store_key: SK, params_subspace_key: PSK, gov_mod: M, bank_keeper: BK) -> Self {
        Self {
            store_key,
            gov_params_keeper: GovParamsKeeper {
                params_subspace_key,
            },
            gov_mod,
            bank_keeper,
            _bank_marker: PhantomData,
        }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        GovGenesisState {
            starting_proposal_id,
            deposits,
            votes,
            proposals,
            params,
        }: GovGenesisState,
    ) {
        {
            let mut store = ctx.kv_store_mut(&self.store_key);
            store.set(PROPOSAL_ID_KEY, starting_proposal_id.to_be_bytes())
        }
        self.gov_params_keeper.set(ctx, params);

        let total_deposits = {
            let mut store_mut = ctx.kv_store_mut(&self.store_key);

            let total_deposits = {
                let mut total_deposits = Vec::with_capacity(deposits.len());
                for deposit in deposits {
                    store_mut.set(
                        MsgDeposit::key(deposit.proposal_id, &deposit.depositor),
                        serde_json::to_vec(&deposit).expect(SERDE_JSON_CONVERSION),
                    ); // TODO:NOW IS THIS CORRECT SERIALIZATION?
                    total_deposits.push(deposit.amount);
                }

                total_deposits.into_iter().flatten().collect::<Vec<_>>()
            };

            for vote in votes {
                store_mut.set(
                    MsgVoteWeighted::key(vote.proposal_id, &vote.voter),
                    serde_json::to_vec(&vote).expect(SERDE_JSON_CONVERSION),
                )
            }

            for proposal in proposals {
                match proposal.status {
                    ProposalStatus::DepositPeriod => {
                        store_mut.set(
                            Proposal::inactive_queue_key(
                                proposal.proposal_id,
                                &proposal.deposit_end_time,
                            ),
                            proposal.proposal_id.to_be_bytes(),
                        );
                    }
                    ProposalStatus::VotingPeriod => store_mut.set(
                        Proposal::active_queue_key(
                            proposal.proposal_id,
                            &proposal.deposit_end_time,
                        ),
                        proposal.proposal_id.to_be_bytes(),
                    ),
                    _ => (),
                }

                store_mut.set(
                    proposal.key(),
                    serde_json::to_vec(&proposal).expect(SERDE_JSON_CONVERSION),
                );
            }

            total_deposits
        };

        let balance = self
            .bank_keeper
            .balance_all(ctx, &self.gov_mod.get_address())
            .unwrap_gas();
        /*
           Okay. I think that in our implementation there is no need to create account if it.

           So I should omit this lines...
           if balance.is_empty() || balance.iter().any(|this| this.amount.is_zero()) {
               https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/genesis.go#L47
           }
        */

        if !(balance == total_deposits) {
            panic!(
                "expected module account was {:?} but we got {:?}",
                balance, total_deposits
            )
        }
    }

    pub fn deposit_add<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        MsgDeposit {
            proposal_id,
            depositor,
            amount,
        }: MsgDeposit,
    ) -> anyhow::Result<bool> {
        let mut proposal = proposal_get(ctx, &self.store_key, proposal_id)?
            .ok_or(anyhow!("unknown proposal {proposal_id}"))?;

        match proposal.status {
            ProposalStatus::DepositPeriod | ProposalStatus::VotingPeriod => Ok(()),
            _ => Err(anyhow!("inactive proposal {proposal_id}")),
        }?;

        self.bank_keeper.send_coins_from_account_to_module(
            ctx,
            depositor.clone(),
            &self.gov_mod,
            amount.clone(),
        )?;

        proposal.total_deposit = proposal.total_deposit.checked_add(amount.clone())?;
        proposal_set(ctx, &self.store_key, &proposal)?;

        let deposit_params = self.gov_params_keeper.try_get(ctx)?.deposit;
        let activated_voting_period = match proposal.status {
            ProposalStatus::DepositPeriod
                if proposal
                    .total_deposit
                    .is_all_gte(&deposit_params.min_deposit) =>
            {
                true
            }
            _ => false,
        };

        let deposit = match deposit_get(ctx, &self.store_key, proposal_id, &depositor)? {
            Some(mut deposit) => {
                deposit.amount = deposit.amount.checked_add(amount)?;
                deposit
            }
            None => MsgDeposit {
                proposal_id,
                depositor,
                amount,
            },
        };

        // TODO: ADD HOOK https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/keeper/deposit.go#L149

        ctx.push_event(Event::new(
            "proposal_deposit",
            vec![
                EventAttribute::new(
                    "amount".into(),
                    format!("{:?}", deposit.amount).into(),
                    false,
                ),
                EventAttribute::new(
                    "proposal_id".into(),
                    format!("{}", deposit.proposal_id).into(),
                    false,
                ),
            ],
        ));

        deposit_set(ctx, &self.store_key, &deposit)?;

        Ok(activated_voting_period)
    }

    pub fn vote_add<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        vote: MsgVoteWeighted,
    ) -> anyhow::Result<()> {
        let proposal = proposal_get(ctx, &self.store_key, vote.proposal_id)?
            .ok_or(anyhow!("unknown proposal {}", vote.proposal_id))?;

        match proposal.status {
            ProposalStatus::VotingPeriod => Ok(()),
            _ => Err(anyhow!("inactive proposal {}", vote.proposal_id)),
        }?;

        vote_set(ctx, &self.store_key, &vote)?;

        // TODO:NOW HOOK https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/keeper/vote.go#L31

        ctx.push_event(Event::new(
            "proposal_vote",
            vec![
                EventAttribute::new("option".into(), format!("{:?}", vote.options).into(), false),
                EventAttribute::new(
                    "proposal_id".into(),
                    format!("{}", vote.proposal_id).into(),
                    false,
                ),
            ],
        ));

        Ok(())
    }

    pub fn submit_proposal<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        MsgSubmitProposal {
            content,
            initial_deposit,
            proposer: _proposer,
        }: MsgSubmitProposal,
    ) -> anyhow::Result<u64> {
        /*
           in go they perform check is it possible to handle
           proposal somehow, but not sure we need it and instead
           handle manually. at least this is such concept at moment

           https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/keeper/proposal.go#L14-L16
        */

        let proposal_id = proposal_id_get(ctx, &self.store_key)?;
        let submit_time = ctx.header().time.clone();
        let deposit_period = self
            .gov_params_keeper
            .try_get(ctx)?
            .deposit
            .max_deposit_period;

        let submit_date =
            DateTime::from_timestamp(submit_time.seconds, submit_time.nanos as u32).unwrap(); // TODO

        let proposal = Proposal {
            proposal_id,
            content,
            status: ProposalStatus::DepositPeriod,
            final_tally_result: Default::default(),
            submit_time: submit_date,
            deposit_end_time: submit_date.add(deposit_period),
            total_deposit: initial_deposit,
            voting_start_time: None,
            voting_end_time: None,
        };

        proposal_set(ctx, &self.store_key, &proposal)?;
        let mut store = ctx.kv_store_mut(&self.store_key);

        store.set(
            Proposal::inactive_queue_key(proposal.proposal_id, &proposal.deposit_end_time),
            proposal.proposal_id.to_be_bytes(),
        )?;

        store.set(PROPOSAL_ID_KEY, (proposal_id + 1).to_be_bytes())?;

        // TODO:NOW HOOK https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/keeper/proposal.go#L45

        ctx.push_event(Event::new(
            "submit_proposal",
            vec![EventAttribute::new(
                "proposal_id".into(),
                proposal_id.to_string().into(),
                false,
            )],
        ));

        Ok(proposal.proposal_id)
    }

    pub fn end_block<DB: Database>(&self, ctx: &mut BlockContext<'_, DB, SK>) {
        let time = DateTime::from_timestamp(ctx.header.time.seconds, ctx.header.time.nanos as u32)
            .unwrap(); // TODO

        {
            let inactive_iter = {
                let store = ctx.kv_store(&self.store_key);
                InactiveProposalIterator::new(&store.into(), &time)
                    .map(|this| this.map(|((proposal_id, _), _)| proposal_id))
                    .collect::<Vec<_>>()
            };

            for var in inactive_iter {
                let proposal_id = var.unwrap_gas();
                proposal_del(ctx, &self.store_key, proposal_id).unwrap_gas();
            }
        }

        {
            let active_iter = {
                let store = ctx.kv_store(&self.store_key).into();
                ActiveProposalIterator::new(&store, &time)
                    .map(|this| this.map(|((proposal_id, _), _)| proposal_id))
                    .collect::<Vec<_>>()
            };

            for var in active_iter {
                let _proposal_id = var.unwrap_gas();
            }
        }
    }
}

fn proposal_id_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
    ctx: &CTX,
    store_key: &SK,
) -> Result<u64, GasStoreErrors> {
    let store = ctx.kv_store(store_key);

    let bytes = store
        .get(PROPOSAL_ID_KEY.as_slice())?
        .expect("Invalid genesis, initial proposal ID hasn't been set");

    Ok(u64::from_be_bytes(
        bytes.try_into().expect("we know it serialized correctly"),
    ))
}

fn proposal_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
    ctx: &CTX,
    store_key: &SK,
    proposal_id: u64,
) -> Result<Option<Proposal>, GasStoreErrors> {
    let key = [KEY_PROPOSAL_PREFIX.as_slice(), &proposal_id.to_be_bytes()].concat();

    let store = ctx.kv_store(store_key);

    let bytes = store.get(&key)?;
    match bytes {
        Some(var) => Ok(Some(
            serde_json::from_slice(&var).expect(SERDE_JSON_CONVERSION),
        )),
        None => Ok(None),
    }
}

fn proposal_set<DB: Database, SK: StoreKey, CTX: TransactionalContext<DB, SK>>(
    ctx: &mut CTX,
    store_key: &SK,
    proposal: &Proposal,
) -> Result<(), GasStoreErrors> {
    let mut store = ctx.kv_store_mut(store_key);

    store.set(
        proposal.key(),
        serde_json::to_vec(proposal).expect(SERDE_JSON_CONVERSION),
    )
}

fn proposal_del<DB: Database, SK: StoreKey, CTX: TransactionalContext<DB, SK>>(
    ctx: &mut CTX,
    store_key: &SK,
    proposal_id: u64,
) -> Result<bool, GasStoreErrors> {
    let proposal = proposal_get(ctx, store_key, proposal_id)?;

    if let Some(proposal) = proposal {
        let mut store = ctx.kv_store_mut(store_key);

        store.delete(&Proposal::inactive_queue_key(
            proposal_id,
            &proposal.deposit_end_time,
        ))?;

        store.delete(&Proposal::active_queue_key(
            proposal_id,
            &proposal.deposit_end_time,
        ))?;

        store.delete(&proposal.key())?;

        Ok(true)
    } else {
        Ok(false)
    }
}

fn deposit_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
    ctx: &CTX,
    store_key: &SK,
    proposal_id: u64,
    depositor: &AccAddress,
) -> Result<Option<MsgDeposit>, GasStoreErrors> {
    let key = [
        MsgDeposit::KEY_PREFIX.as_slice(),
        &proposal_id.to_be_bytes(),
        &[depositor.len()],
        depositor.as_ref(),
    ]
    .concat();

    let store = ctx.kv_store(store_key);

    let bytes = store.get(&key)?;
    match bytes {
        Some(var) => Ok(Some(
            serde_json::from_slice(&var).expect(SERDE_JSON_CONVERSION),
        )),
        None => Ok(None),
    }
}

fn deposit_set<DB: Database, SK: StoreKey, CTX: TransactionalContext<DB, SK>>(
    ctx: &mut CTX,
    store_key: &SK,
    deposit: &MsgDeposit,
) -> Result<(), GasStoreErrors> {
    let mut store = ctx.kv_store_mut(store_key);

    store.set(
        MsgDeposit::key(deposit.proposal_id, &deposit.depositor),
        serde_json::to_vec(deposit).expect(SERDE_JSON_CONVERSION),
    )
}

fn _vote_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
    ctx: &CTX,
    store_key: &SK,
    proposal_id: u64,
    voter: &AccAddress,
) -> Result<Option<MsgVoteWeighted>, GasStoreErrors> {
    let key = [
        MsgVoteWeighted::KEY_PREFIX.as_slice(),
        &proposal_id.to_be_bytes(),
        &[voter.len()],
        voter.as_ref(),
    ]
    .concat();

    let store = ctx.kv_store(store_key);

    let bytes = store.get(&key)?;
    match bytes {
        Some(var) => Ok(Some(
            serde_json::from_slice(&var).expect(SERDE_JSON_CONVERSION),
        )),
        None => Ok(None),
    }
}

fn vote_set<DB: Database, SK: StoreKey, CTX: TransactionalContext<DB, SK>>(
    ctx: &mut CTX,
    store_key: &SK,
    vote: &MsgVoteWeighted,
) -> Result<(), GasStoreErrors> {
    let mut store = ctx.kv_store_mut(store_key);

    store.set(
        MsgVoteWeighted::key(vote.proposal_id, &vote.voter),
        serde_json::to_vec(vote).expect(SERDE_JSON_CONVERSION),
    )
}
