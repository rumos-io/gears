use bytes::Bytes;
use ibc_proto::cosmos::tx::v1beta1::{AuthInfo, TxBody};

/// Nutype struct for body bytes
#[derive(Debug)]
pub struct BodyBytes(pub Bytes);
/// Nutype struct for auth info bytes
#[derive(Debug)]
pub struct AuthBytes(pub Bytes);

/// TxData is the data about a transaction that is necessary to generate sign bytes.
#[derive(Debug)]
pub struct TxData {
    /// `body` is the `TxBody` that will be part of the transaction.
    pub body: TxBody,

    /// `auth_info` is the `AuthInfo` that will be part of the transaction.
    pub auth_info: AuthInfo, // Same here

    /// `body_bytes` is the marshaled body bytes that will be part of `TxRaw`.
    pub body_bytes: BodyBytes,

    /// `auth_info_bytes` is the marshaled `AuthInfo` bytes that will be part of `TxRaw`.
    pub auth_info_bytes: AuthBytes,

    /// BodyHasUnknownNonCriticals should be set to true if the transaction has been
    /// decoded and found to have unknown non-critical fields. This is only needed
    /// for amino JSON signing.
    pub body_has_unknown_non_criticals: bool,
}
