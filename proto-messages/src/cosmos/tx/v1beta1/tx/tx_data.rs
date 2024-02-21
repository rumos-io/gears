use serde::{Deserialize, Serialize};

use crate::cosmos::tx::v1beta1::message::Message;

use super::{auth_info::AuthInfo, tx_body::TxBody};

/// TxData is the data about a transaction that is necessary to generate sign bytes.
#[derive(Debug, Serialize, Deserialize)]
pub struct TxData<M: Message> {
    /// `body` is the `TxBody` that will be part of the transaction.
    pub body: TxBody<M>,

    /// `auth_info` is the `AuthInfo` that will be part of the transaction.
    pub auth_info: AuthInfo, // Same here
}
