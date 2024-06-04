use std::time::Duration;

use gears::types::base::coin::Coin;

// pub struct GovParams {
//     /// Minimum deposit for a proposal to enter voting period.
//     pub min_deposit: Vec<Coin>, //  send::SendCoins?
//     /// Maximum period for stake holders to deposit on a proposal. Initial value: 2 months.
//     pub max_deposit_duration: Duration, // Std or any other?
//     /// Duration of the voting period.
//     pub voting_period: Duration, // Std or any other?
//     /// Minimum percentage of total stake needed to vote for a result to be considered valid.
//     pub quorum: String,
//     /// Minimum proportion of Yes votes for proposal to pass. Default value: 0.5.
//     pub threshold: String,
//     ///  Minimum value of Veto votes to Total votes ratio for proposal to be vetoed. Default value: 1/3.
//     pub veto_threshold: String,
//     /// The ratio representing the proportion of the deposit value that must be paid at proposal submission.
//     pub min_initial_deposit_ratio: String,
//     /// The cancel ratio which will not be returned back to the depositors when a proposal is cancelled.
//     pub proposal_cancel_ratio: String,
//     /// The address which will receive (proposal_cancel_ratio * deposit) proposal deposits.
//     /// If empty, the (proposal_cancel_ratio * deposit) proposal deposits will be burned.
//     pub proposal_cancel_cest: String,
//     /// Duration of the voting period of an expedited proposal.
//     pub expedited_voting_period: Duration,
//     /// Minimum proportion of Yes votes for proposal to pass. Default value: 0.67.
//     pub expedited_threshold: String,
//     /// Minimum expedited deposit for a proposal to enter voting period.
//     pub expedited_min_deposit: Vec<Coin>, //  send::SendCoins?
//     /// burn deposits if a proposal does not meet quorum
//     pub burn_vote_quorum: bool,
//     /// burn deposits if the proposal does not enter voting period
//     pub burn_proposal_deposit_prevote: bool,
//     /// burn deposits if quorum with vote type no_veto is met
//     pub burn_vote_veto: bool,
//     /// The ratio representing the proportion of the deposit value minimum that must be met when making a deposit.
//     /// Default value: 0.01. Meaning that for a chain with a min_deposit of 100stake, a deposit of 1stake would be
//     /// required.
//     pub min_deposit_ratio: String,
//     /// proposal_cancel_max_period defines how far in the voting period a proposer can cancel a proposal.
//     /// If the proposal is cancelled before the max cancel period, the deposit will be returned/burn to the
//     /// depositors, according to the proposal_cancel_ratio and proposal_cancel_dest parameters.
//     /// After the max cancel period, the proposal cannot be cancelled anymore.
//     pub proposal_cancel_max_period: String,
//     /// optimistic_authorized_addresses is an optional governance parameter that limits
//     /// the authorized accounts than can submit optimistic proposals
//     pub optimistic_authorized_addresses: Vec<String>,
//     /// optimistic rejected threshold defines at which percentage of NO votes, the optimistic proposal should fail and be
//     /// converted to a standard proposal. The threshold is expressed as a percentage of the total bonded tokens.
//     pub optimistic_rejected_threshold: String,
//     /// yes_quorum defines the minimum percentage of Yes votes in quorum for proposal to pass.
//     /// Default value: 0 (disabled).
//     pub yes_quorum: String,
//     /// Minimum percentage of total stake needed to vote for a result to be
//     /// considered valid for an expedited proposal.
//     pub expedited_quorum: String,
// }

pub struct DepositParams {
    pub min_deposit: Vec<Coin>, // SendCoins
    /// Duration of the voting period.
    pub voting_period: Duration, // Std or any other?
}

pub struct VotionParams
{
    
}