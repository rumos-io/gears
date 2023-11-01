use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignatureData {
    pub signature: Vec<u8>,
    pub sequence: u64,
}
