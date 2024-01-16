use serde::{Deserialize, Serialize};

use crate::cosmos::tx::v1beta1::message::Message;

use super::tx::Tx;

/// This enum is used where a Tx needs to be serialized like an Any
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum AnyTx<M: Message> {
    #[serde(rename = "/cosmos.tx.v1beta1.Tx")]
    Tx(Tx<M>),
}
