use std::sync::OnceLock;

use chrono::{DateTime, SubsecRound, Utc};
use gears::types::base::send::SendCoins;
use ibc_proto::google::protobuf::Any;
use serde::{Deserialize, Serialize};

use crate::keeper::KEY_PROPOSAL_PREFIX;

pub mod active_iter;
pub mod inactive_iter;

// Slight modification of the RFC3339Nano but it right pads all zeros and drops the time zone info
const SORTABLE_DATE_TIME_FORMAT: &str = "%Y-%m-%dT&H:%M:%S.000000000";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub proposal_id: u64,
    pub content: Any,
    pub status: ProposalStatus,
    pub final_tally_result: TallyResult,
    pub submit_time: DateTime<Utc>,
    pub deposit_end_time: DateTime<Utc>,
    pub total_deposit: SendCoins,
    pub voting_start_time: Option<DateTime<Utc>>,
    pub voting_end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TallyResult {
    yes: i32,
    abstain: i32,
    no: i32,
    no_with_veto: i32,
}

impl Proposal {
    const KEY_ACTIVE_QUEUE_PREFIX: [u8; 1] = [0x01];
    const KEY_INACTIVE_QUEUE_PREFIX: [u8; 1] = [0x02];

    pub fn key(&self) -> Vec<u8> {
        [
            KEY_PROPOSAL_PREFIX.as_slice(),
            &self.proposal_id.to_be_bytes(),
        ]
        .concat()
    }

    pub fn inactive_queue_key(proposal_id: u64, deposit_end_time: &DateTime<Utc>) -> Vec<u8> {
        Self::queue_key(
            &Self::KEY_INACTIVE_QUEUE_PREFIX,
            proposal_id,
            deposit_end_time,
        )
    }

    pub fn active_queue_key(proposal_id: u64, deposit_end_time: &DateTime<Utc>) -> Vec<u8> {
        Self::queue_key(
            &Self::KEY_ACTIVE_QUEUE_PREFIX,
            proposal_id,
            deposit_end_time,
        )
    }

    fn queue_key(prefix: &[u8], proposal_id: u64, deposit_end_time: &DateTime<Utc>) -> Vec<u8> {
        let date_key = deposit_end_time
            .round_subsecs(0)
            .format(SORTABLE_DATE_TIME_FORMAT)
            .to_string();

        [prefix, date_key.as_bytes(), &proposal_id.to_be_bytes()].concat()
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

fn parse_proposal_key_bytes(bytes: impl AsRef<[u8]>) -> (u64, DateTime<Utc>) {
    static KEY_LENGTH: OnceLock<usize> = OnceLock::new();

    let length_time = *KEY_LENGTH.get_or_init(|| {
        Utc::now()
            .round_subsecs(0)
            .format(SORTABLE_DATE_TIME_FORMAT)
            .to_string()
            .bytes()
            .len()
    });

    let time = DateTime::parse_from_rfc3339(
        &String::from_utf8(bytes.as_ref()[1..1 + length_time].to_vec())
            .expect("We serialize date as String so conversion is save"),
    )
    .unwrap() // TODO
    .to_utc();
    let proposal = u64::from_be_bytes(bytes.as_ref()[1 + length_time..].try_into().unwrap());
    // TODO

    (proposal, time)
}
