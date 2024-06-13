use gears::types::{address::AccAddress, base::send::SendCoins};
use serde::{Deserialize, Serialize};

use crate::keeper::KEY_DEPOSIT_PREFIX;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deposit {
    pub proposal_id: u64,
    pub depositor: AccAddress,
    pub amount: SendCoins,
}

impl Deposit {
    pub(crate) fn key(&self) -> Vec<u8> {
        [
            KEY_DEPOSIT_PREFIX.as_slice(),
            &self.proposal_id.to_be_bytes(),
            &[self.depositor.len()],
            self.depositor.as_ref(),
        ]
        .concat()
    }
}
