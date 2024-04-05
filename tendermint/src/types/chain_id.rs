// Copy of https://docs.rs/ibc-core-host-types/0.51.0/src/ibc_core_host_types/identifiers/chain_id.rs.html#33

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

use super::validate::{
    validate_identifier_chars, validate_identifier_length, validate_prefix_length,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum ChainIdErrors {
    #[error("revision number overflowed")]
    RevisionNumberOverflow,
    #[error("chain identifier: {0} is not formatted with revision number")]
    UnformattedRevisionNumber(String),
    #[error(
        "identifier `{id}` has invalid length; must be between `{min}` and `{max}` characters"
    )]
    InvalidLength { id: String, min: u64, max: u64 },
    #[error("identifier `{0}` must only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`")]
    InvalidCharacter(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainId {
    id: String,
    revision_number: u64,
}

impl From<tendermint_informal::chain::Id> for ChainId {
    fn from(value: tendermint_informal::chain::Id) -> Self {
        Self::from_str(value.as_str()).expect("Structures are same")
    }
}

impl From<ChainId> for tendermint_informal::chain::Id {
    fn from(value: ChainId) -> Self {
        Self::from_str(value.as_str()).expect("Structures are same")
    }
}

impl ChainId {
    /// Creates a new `ChainId` with the given chain identifier.
    ///
    /// It checks the identifier for valid characters according to `ICS-24`
    /// specification and returns a `ChainId` successfully.
    /// Stricter checks beyond `ICS-24` rests with the users,
    /// based on their requirements.
    ///
    /// If the chain identifier is in the {chain name}-{revision number} format,
    /// the revision number is parsed. Otherwise, revision number is set to 0.
    pub fn new(chain_id: &str) -> Result<Self, ChainIdErrors> {
        Self::from_str(chain_id)
    }

    /// Get a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.id
    }

    pub fn split_chain_id(&self) -> Result<(&str, u64), ChainIdErrors> {
        parse_chain_id_string(self.as_str())
    }

    /// Extract the revision number from the chain identifier
    pub fn revision_number(&self) -> u64 {
        self.revision_number
    }

    /// Increases `ChainId`s revision number by one.
    /// Fails if the chain identifier is not in
    /// `{chain_name}-{revision_number}` format or
    /// the revision number overflows.
    pub fn increment_revision_number(&mut self) -> Result<(), ChainIdErrors> {
        let (chain_name, _) = self.split_chain_id()?;
        let inc_revision_number = self
            .revision_number
            .checked_add(1)
            .ok_or(ChainIdErrors::RevisionNumberOverflow)?;
        self.id = format!("{}-{}", chain_name, inc_revision_number);
        self.revision_number = inc_revision_number;
        Ok(())
    }

    /// A convenient method to check if the `ChainId` forms a valid identifier
    /// with the desired min/max length. However, ICS-24 does not specify a
    /// certain min or max lengths for chain identifiers.
    pub fn validate_length(&self, min_length: u64, max_length: u64) -> Result<(), ChainIdErrors> {
        match self.split_chain_id() {
            Ok((chain_name, _)) => validate_prefix_length(chain_name, min_length, max_length),
            _ => validate_identifier_length(&self.id, min_length, max_length),
        }
    }
}

impl<'de> Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["id", "revision_number"];

        enum Field {
            Id,
            RevisionNumber,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "id" => Ok(Field::Id),
                            "revisionNumber" | "revision_number" => Ok(Field::RevisionNumber),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ChainIdVisitor;

        impl<'de> Visitor<'de> for ChainIdVisitor {
            type Value = ChainId;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ChainId")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut revision_number = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(Error::duplicate_field("id"));
                            }

                            let next_value = map.next_value::<&str>()?;

                            let chain_id = ChainId::from_str(next_value)
                                .map_err(|_| Error::custom("failed to parse ChainId `id` field"))?;

                            id = Some(chain_id.id);
                            revision_number = Some(chain_id.revision_number);
                        }
                        Field::RevisionNumber => {
                            let next_value = map.next_value::<&str>()?;
                            let rev = u64::from_str(next_value).unwrap_or(0);

                            if let Some(rn) = revision_number {
                                if rev != 0 && rn != rev {
                                    return Err(Error::custom(format_args!(
                                        "chain ID revision numbers do not match; got `{}` and `{}`",
                                        rn, rev,
                                    )));
                                }
                            } else {
                                revision_number = Some(rev);
                            }
                        }
                    }
                }

                let id = id.ok_or_else(|| Error::missing_field("id"))?;

                Ok(ChainId {
                    id,
                    revision_number: revision_number.unwrap_or(0),
                })
            }
        }

        deserializer.deserialize_struct("ChainId", FIELDS, ChainIdVisitor)
    }
}

/// Construct a `ChainId` from a string literal only if it forms a valid
/// identifier.
impl FromStr for ChainId {
    type Err = ChainIdErrors;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        // Identifier string must have a maximum length of 64 characters.

        // Validates the chain name for allowed characters according to ICS-24.
        validate_identifier_chars(id)?;
        if let Ok((chain_name, revision_number)) = parse_chain_id_string(id) {
            // Validate if the chain name with revision number has a valid length.
            validate_prefix_length(chain_name, 1, 64)?;
            Ok(Self {
                id: id.into(),
                revision_number,
            })
        } else {
            // Validate if the identifier has a valid length.
            validate_identifier_length(id, 1, 64)?;
            Ok(Self {
                id: id.into(),
                revision_number: 0,
            })
        }
    }
}

impl From<ChainId> for String {
    fn from(chain_id: ChainId) -> String {
        chain_id.id
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.id)
    }
}

/// Parses a string intended to represent a `ChainId` and, if successful,
/// returns a tuple containing the chain name and revision number.
fn parse_chain_id_string(chain_id_str: &str) -> Result<(&str, u64), ChainIdErrors> {
    chain_id_str
        .rsplit_once('-')
        .filter(|(_, rev_number_str)| {
            // Validates the revision number not to start with leading zeros, like "01".
            // Zero is the only allowed revision number with leading zero.
            rev_number_str.as_bytes().first() != Some(&b'0') || rev_number_str.len() == 1
        })
        .and_then(|(chain_name, rev_number_str)| {
            // Parses the revision number string into a `u64` and checks its validity.
            rev_number_str
                .parse()
                .ok()
                .map(|revision_number| (chain_name, revision_number))
        })
        .ok_or(ChainIdErrors::UnformattedRevisionNumber(
            chain_id_str.to_string(),
        ))
}
