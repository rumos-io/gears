use gears::types::{address::AccAddress, base::send::SendCoins};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deposit {
    pub proposal_id: u64,
    pub depositor: AccAddress,
    pub amount: SendCoins,
}

impl Deposit {
    pub(crate) const KEY_PREFIX: [u8; 1] = [0x10];

    pub(crate) fn key(proposal_id: u64, depositor: &AccAddress) -> Vec<u8> {
        [
            Self::KEY_PREFIX.as_slice(),
            &proposal_id.to_be_bytes(),
            &[depositor.len()],
            depositor.as_ref(),
        ]
        .concat()
    }
}
