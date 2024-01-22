use std::str::FromStr;

pub use bnum::types::U256 as BU256;
use cosmwasm_std::StdError;
use schemars::JsonSchema;
use serde::Serialize;

// Simplified replica of https://docs.rs/cosmwasm-std/latest/src/cosmwasm_std/math/uint256.rs.html#49

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct U256(#[schemars(with = "String")] pub BU256);

impl U256 {
    pub fn into_inner(self) -> BU256 {
        self.0
    }
}

impl From<BU256> for U256 {
    fn from(value: BU256) -> Self {
        Self(value)
    }
}

impl core::fmt::Display for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<U256> for String {
    fn from(original: U256) -> Self {
        original.to_string()
    }
}

impl FromStr for U256 {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(StdError::generic_err("Parsing u256: received empty string"));
        }

        match BU256::from_str_radix(s, 10) {
            Ok(u) => Ok(U256(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing u256: {e}"))),
        }
    }
}

impl TryFrom<&str> for U256 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl Serialize for U256 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for U256 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint256Visitor)
    }
}

struct Uint256Visitor;

impl<'de> serde::de::Visitor<'de> for Uint256Visitor {
    type Value = U256;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        U256::try_from(v).map_err(|e| E::custom(format!("invalid Uint256 '{v}' - {e}")))
    }
}
