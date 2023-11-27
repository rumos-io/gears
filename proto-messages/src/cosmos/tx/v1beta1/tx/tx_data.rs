use prost::bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::cosmos::tx::v1beta1::message::Message;

use super::{auth_info::AuthInfo, tx_body::TxBody};

/// Nutype struct for body bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyBytes(pub Bytes);
/// Nutype struct for auth info bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthBytes(pub Bytes);

/// TxData is the data about a transaction that is necessary to generate sign bytes.
#[derive(Debug, Serialize, Deserialize)]
pub struct TxData<M: Message> {
    /// `body` is the `TxBody` that will be part of the transaction.
    pub body: TxBody<M>,

    /// `auth_info` is the `AuthInfo` that will be part of the transaction.
    pub auth_info: AuthInfo, // Same here

    /// BodyHasUnknownNonCriticals should be set to true if the transaction has been
    /// decoded and found to have unknown non-critical fields. This is only needed
    /// for amino JSON signing.
    pub body_has_unknown_non_criticals: bool,
}
