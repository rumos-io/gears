use gears::types::address::AccAddress;
use serde::{Deserialize, Serialize};

use crate::keeper::KEY_VOTES_PREFIX;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub options: (), // TODO:
}

impl Vote {
    pub fn key(&self) -> Vec<u8> {
        [
            KEY_VOTES_PREFIX.as_slice(),
            &self.proposal_id.to_be_bytes(),
            &[self.voter.len() as u8], // We save 'cause `AccAddress` len shoudn't be bigger than 255
            self.voter.as_ref(),
        ]
        .concat()
    }
}
