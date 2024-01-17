use ibc_proto::{cosmos::tx::v1beta1::TxRaw, protobuf::Protobuf};
use prost::{bytes::Bytes, Message as ProstMessage};

use serde::{Deserialize, Serialize};

use crate::{cosmos::tx::v1beta1::message::Message, error::Error};

use super::tx::Tx;

/// Tx is the standard type used for broadcasting transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TxWithRaw<M: Message> {
    pub tx: Tx<M>,
    pub raw: TxRaw,
}

impl<M: Message> TxWithRaw<M> {
    pub fn from_bytes(raw: Bytes) -> Result<Self, Error> {
        let tx = Tx::decode(raw.clone()).map_err(|e| Error::DecodeGeneral(format!("{}", e)))?;

        let raw = TxRaw::decode(raw).map_err(|e| Error::DecodeGeneral(format!("{}", e)))?;
        Ok(TxWithRaw { tx, raw })
    }
}
