use bytes::Bytes;
use nutype::nutype;
use prost::Message;
use proto_types::Denom;
use serde::{Deserialize, Serialize};
use tendermint::types::proto::Protobuf;

mod inner {
    pub use ibc_proto::tx::denom::DenomUnit;
}

/// We use our own version of the Metadata struct because the one in ibc_proto
/// has additional fields that were added in SDK v46 (uri and uri_hash). If we
/// don't exclude them then we won't arrive at the same state hash as v45
/// chains.
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawMetadata {
    #[prost(string, tag = "1")]
    description: String,
    #[prost(message, repeated, tag = "2")]
    denom_units: Vec<inner::DenomUnit>,
    #[prost(string, tag = "3")]
    base: String,
    #[prost(string, tag = "4")]
    display: String,
    #[prost(string, tag = "5")]
    name: String,
    #[prost(string, tag = "6")]
    symbol: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct DenomUnit {
    pub denom: Denom,
    pub exponent: u32,
    pub aliases: Vec<String>,
}

impl TryFrom<inner::DenomUnit> for DenomUnit {
    type Error = proto_types::error::Error;

    fn try_from(
        inner::DenomUnit {
            denom,
            exponent,
            aliases,
        }: inner::DenomUnit,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: Denom::try_from(denom)?,
            exponent,
            aliases,
        })
    }
}

impl From<DenomUnit> for inner::DenomUnit {
    fn from(
        DenomUnit {
            denom,
            exponent,
            aliases,
        }: DenomUnit,
    ) -> Self {
        Self {
            denom: denom.into_inner(),
            exponent,
            aliases,
        }
    }
}

#[nutype(validate(not_empty), derive(Debug, Clone, Deserialize, Serialize))]
pub struct UriHash(String);

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub description: String,
    /// denom_units represents the list of DenomUnit's for a given coin
    pub denom_units: Vec<DenomUnit>,
    /// base represents the base denom (should be the DenomUnit with exponent = 0).
    pub base: String,
    /// display indicates the suggested denom that should be
    /// displayed in clients.
    pub display: String,
    /// name defines the name of the token (eg: Cosmos Atom)
    pub name: String,
    /// symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
    /// be the same as the display.
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("Error parsing: {0}")]
pub struct MetadataParseError(pub String);

impl Metadata {
    pub fn from_bytes(raw: Bytes) -> Result<Self, MetadataParseError> {
        let meta = RawMetadata::decode(raw).map_err(|e| MetadataParseError(e.to_string()))?;

        meta.try_into()
    }
}

impl Protobuf<RawMetadata> for Metadata {}

impl TryFrom<RawMetadata> for Metadata {
    type Error = MetadataParseError;

    fn try_from(value: RawMetadata) -> Result<Self, Self::Error> {
        let RawMetadata {
            description,
            denom_units,
            base,
            display,
            name,
            symbol,
        } = value;

        let mut mapped_denom: Vec<DenomUnit> = Vec::with_capacity(denom_units.len());
        for unit in denom_units {
            mapped_denom.push(
                DenomUnit::try_from(unit)
                    .map_err(|e: proto_types::error::Error| MetadataParseError(e.to_string()))?,
            );
        }

        Ok(Self {
            description,
            denom_units: mapped_denom,
            base,
            display,
            name,
            symbol,
        })
    }
}

impl From<Metadata> for RawMetadata {
    fn from(value: Metadata) -> Self {
        let Metadata {
            description,
            denom_units,
            base,
            display,
            name,
            symbol,
        } = value;

        Self {
            description,
            denom_units: denom_units
                .into_iter()
                .map(inner::DenomUnit::from)
                .collect(),
            base,
            display,
            name,
            symbol,
        }
    }
}
