use chrono::{DateTime, SubsecRound, Utc};
use gears::types::base::coin::Coin;
use serde::{Deserialize, Serialize};

use crate::keeper::KEY_PROPOSAL_PREFIX;

// Slight modification of the RFC3339Nano but it right pads all zeros and drops the time zone info
const SORTABLE_DATE_TIME_FORMAT: &str = "%Y-%m-%dT&H:%M:%S.000000000"; // TODO:NOW Is %S for seconds?

const KEY_ACTIVE_PROPOSAL_QUEUE_PREFIX: [u8; 1] = [0x01];
const KEY_INACTIVE_PROPOSAL_QUEUE_PREFIX: [u8; 1] = [0x02];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub proposal_id: u64,
    pub content: Vec<u8>, // TODO
    pub status: ProposalStatus,
    pub final_tally_result: (), // TODO: https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/gov/types/gov.pb.go#L289
    pub submit_time: (),
    pub deposit_end_time: DateTime<Utc>,
    pub total_deposit: Vec<Coin>,
    pub voting_start_time: (),
    pub voting_end_time: DateTime<Utc>,
}

impl Proposal {
    pub fn key(&self) -> Vec<u8> {
        [
            KEY_PROPOSAL_PREFIX.as_slice(),
            &self.proposal_id.to_be_bytes(),
        ]
        .concat()
    }

    pub fn inactive_queue_key(&self) -> Vec<u8> {
        self.queue_key(&KEY_INACTIVE_PROPOSAL_QUEUE_PREFIX)
    }

    pub fn active_queue_key(&self) -> Vec<u8> {
        self.queue_key(&KEY_ACTIVE_PROPOSAL_QUEUE_PREFIX)
    }

    fn queue_key(&self, prefix: &[u8]) -> Vec<u8> {
        let date_key = self
            .deposit_end_time
            .round_subsecs(0)
            .format(SORTABLE_DATE_TIME_FORMAT)
            .to_string();

        [prefix, date_key.as_bytes(), &self.proposal_id.to_be_bytes()].concat()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Nil,
    DepositPeriod,
    VotingPeriod,
    Passed,
    Rejected,
    Failed,
}