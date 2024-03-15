use ibc_proto::cosmos::bank::v1beta1::DenomUnit as RawDenomUnit;
use ibc_proto::Protobuf;
use nutype::nutype;
use prost::bytes::Bytes;
use prost::Message;
use proto_types::Denom;
use serde::Deserialize;
use serde::Serialize;

use crate::Error;

/// We use our own version of the Metadata struct because the one in ibc_proto
/// has additional fields that were added in SDK v46 (uri and uri_hash). If we
/// don't exclude them then we won't arrive at the same state hash as v45
/// chains.
#[derive(Clone, PartialEq, prost::Message)]
pub struct RawMetadata {
    #[prost(string, tag = "1")]
    description: String,
    #[prost(message, repeated, tag = "2")]
    denom_units: Vec<RawDenomUnit>,
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

impl TryFrom<RawDenomUnit> for DenomUnit {
    type Error = proto_types::Error;

    fn try_from(value: RawDenomUnit) -> Result<Self, Self::Error> {
        let RawDenomUnit {
            denom,
            exponent,
            aliases,
        } = value;

        Ok(Self {
            denom: Denom::try_from(denom)?,
            exponent,
            aliases,
        })
    }
}

impl From<DenomUnit> for RawDenomUnit {
    fn from(value: DenomUnit) -> Self {
        let DenomUnit {
            denom,
            exponent,
            aliases,
        } = value;

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

impl Metadata {
    pub fn from_bytes(raw: Bytes) -> Result<Self, Error> {
        let meta = RawMetadata::decode(raw)?;

        meta.try_into()
    }
}

impl Protobuf<RawMetadata> for Metadata {}

impl TryFrom<RawMetadata> for Metadata {
    type Error = Error;

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
                    .map_err(|e: proto_types::Error| Error::Custom(e.to_string()))?,
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
            denom_units: denom_units.into_iter().map(RawDenomUnit::from).collect(),
            base,
            display,
            name,
            symbol,
        }
    }
}
