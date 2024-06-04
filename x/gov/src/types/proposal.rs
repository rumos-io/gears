use gears::types::base::coin::Coin;

pub struct Proposal {
    pub proposal_id: u64,
    pub content: Vec<u8>, // TODO
    pub status: ProposalStatus,
    pub final_tally_result: (), // TODO: https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/types/gov.pb.go#L289
    pub submit_time: (),
    pub deposit_end_time: (),
    pub total_deposit: Vec<Coin>,
    pub voting_start_time: (),
    pub voting_end_time: (),
}

pub enum ProposalStatus {
    Nil,
    DepositPeriod,
    VotingPeriod,
    Passed,
    Rejected,
    Failed,
}
