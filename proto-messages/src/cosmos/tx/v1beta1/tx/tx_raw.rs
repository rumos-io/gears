use ibc_proto::{cosmos::tx::v1beta1::TxRaw as RawTxRaw, Protobuf};
use prost::{bytes::Bytes, Message as ProstMessage};

use serde::{Deserialize, Serialize};

use crate::{cosmos::tx::v1beta1::message::Message, error::Error};

use super::tx::Tx;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxRaw {
    pub body_bytes: Vec<u8>,
    pub auth_info_bytes: Vec<u8>,
    pub signatures: Vec<Vec<u8>>,
}

impl From<RawTxRaw> for TxRaw {
    fn from(value: RawTxRaw) -> Self {
        let RawTxRaw {
            body_bytes,
            auth_info_bytes,
            signatures,
        } = value;

        Self {
            body_bytes,
            auth_info_bytes,
            signatures,
        }
    }
}

impl From<TxRaw> for RawTxRaw {
    fn from(value: TxRaw) -> Self {
        let TxRaw {
            body_bytes,
            auth_info_bytes,
            signatures,
        } = value;

        Self {
            body_bytes,
            auth_info_bytes,
            signatures,
        }
    }
}

/// Tx is the standard type used for broadcasting transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TxWithRaw<M: Message> {
    pub tx: Tx<M>,
    pub raw: TxRaw,
}

impl<M: Message> TxWithRaw<M> {
    pub fn from_bytes(raw: Bytes) -> Result<Self, Error> {
        let tx = Tx::decode(raw.clone()).map_err(|e| Error::DecodeGeneral(format!("{}", e)))?;

        let raw = RawTxRaw::decode(raw).map_err(|e| Error::DecodeGeneral(format!("{}", e)))?;
        Ok(TxWithRaw {
            tx,
            raw: raw.into(),
        })
    }
}
