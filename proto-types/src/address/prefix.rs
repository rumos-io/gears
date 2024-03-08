use std::{fmt, str::FromStr};

use serde::de::Visitor;

use super::BaseAddress;

pub trait AddressPrefix {
    const PREFIX: &'static str;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Acc;

impl<'de> Visitor<'de> for Acc {
    type Value = BaseAddress<Acc>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bech32 encoded address")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        BaseAddress::from_str(v).map_err(|e| E::custom(format!("invalid address '{}' - {}", v, e)))
    }
}

impl AddressPrefix for Acc {
    const PREFIX: &'static str = "cosmos";
}

#[derive(Debug, PartialEq, Clone)]
pub struct Val;

impl AddressPrefix for Val {
    const PREFIX: &'static str = "cosmosvaloper";
}

impl<'de> Visitor<'de> for Val {
    type Value = BaseAddress<Acc>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bech32 encoded address")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        BaseAddress::from_str(v).map_err(|e| E::custom(format!("invalid address '{}' - {}", v, e)))
    }
}
