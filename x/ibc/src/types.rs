pub use ibc::core::host::types::identifiers::ClientId as RawClientId;
pub use ibc::primitives::Signer as RawSigner;

#[derive(
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct ClientId(pub String);

impl From<&str> for ClientId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Signer(pub String);

impl From<&str> for Signer {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
