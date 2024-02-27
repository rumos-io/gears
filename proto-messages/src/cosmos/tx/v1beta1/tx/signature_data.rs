use serde::{Deserialize, Serialize};

use super::mode_info::ModeInfo;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignatureData {
    pub signature: Vec<u8>,
    pub sequence: u64,
    pub mode_info: ModeInfo,
}
