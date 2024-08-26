use core_types::{errors::CoreError, Protobuf};
use prost::{bytes::Bytes, Message as ProstMessage};

use serde::{Deserialize, Serialize};

use super::{Tx, TxMessage};

mod inner {
    pub use core_types::tx::raw::TxRaw;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxRaw {
    pub body_bytes: Vec<u8>,
    pub auth_info_bytes: Vec<u8>,
    pub signatures: Vec<Vec<u8>>,
}

impl<'a, M: TxMessage> From<&'a Tx<M>> for TxRaw {
    fn from(
        Tx {
            body,
            auth_info,
            signatures,
            signatures_data: _,
        }: &'a Tx<M>,
    ) -> Self {
        Self {
            body_bytes: body.encode_vec(),
            auth_info_bytes: auth_info.encode_vec(),
            signatures: signatures.clone(),
        }
    }
}

impl From<inner::TxRaw> for TxRaw {
    fn from(
        inner::TxRaw {
            body_bytes,
            auth_info_bytes,
            signatures,
        }: inner::TxRaw,
    ) -> Self {
        Self {
            body_bytes,
            auth_info_bytes,
            signatures,
        }
    }
}

impl From<TxRaw> for inner::TxRaw {
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
pub struct TxWithRaw<M> {
    pub tx: Tx<M>,
    pub raw: TxRaw,
    pub tx_len: usize,
}

impl<M: TxMessage> TxWithRaw<M> {
    pub fn from_bytes(raw: Bytes) -> Result<Self, CoreError> {
        let tx_len = raw.len();

        let tx = Tx::decode(raw.clone()).map_err(|e| CoreError::DecodeGeneral(format!("{}", e)))?;

        let raw =
            inner::TxRaw::decode(raw).map_err(|e| CoreError::DecodeGeneral(format!("{}", e)))?;
        Ok(TxWithRaw {
            tx,
            raw: raw.into(),
            tx_len,
        })
    }
}
