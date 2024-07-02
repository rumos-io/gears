use serde::{Deserialize, Serialize};

use crate::types::auth::info::AuthInfo;

use super::{body::TxBody, TxMessage};

/// TxData is the data about a transaction that is necessary to generate sign bytes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxData<M: TxMessage> {
    /// `body` is the `TxBody` that will be part of the transaction.
    pub body: TxBody<M>,

    /// `auth_info` is the `AuthInfo` that will be part of the transaction.
    pub auth_info: AuthInfo, // Same here
}
