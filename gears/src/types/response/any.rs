use serde::{Deserialize, Serialize};

use crate::types::tx::{Tx, TxMessage};

/// This enum is used where a Tx needs to be serialized like an Any
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum AnyTx<M: TxMessage> {
    #[serde(rename = "/cosmos.tx.v1beta1.Tx")]
    Tx(Tx<M>),
}
