use std::{
    fmt::{self, Display},
    str::FromStr,
};

use bech32::{self, FromBase32, ToBase32, Variant};
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::AddressError;

#[derive(Debug, PartialEq, Clone)]
pub struct AccAddress(Vec<u8>);

const ACCOUNT_ADDRESS_PREFIX: &str = env!("ACCOUNT_ADDRESS_PREFIX");
const MAX_ADDR_LEN: u8 = 255;

impl AccAddress {
    pub fn from_bech32(address: &str) -> Result<Self, AddressError> {
        let (hrp, data, variant) = bech32::decode(address)?;

        if hrp != ACCOUNT_ADDRESS_PREFIX {
            return Err(AddressError::InvalidPrefix {
                expected: ACCOUNT_ADDRESS_PREFIX.into(),
                found: hrp,
            });
        };

        if let Variant::Bech32m = variant {
            return Err(AddressError::InvalidVariant {
                expected: "Bech32".into(),
                found: "Bech32m".into(),
            });
        }

        // It's unclear whether the conversion from base32 can ever fail. Since this method
        // already returns a Result there's no harm in returning an error here.
        let address = Vec::<u8>::from_base32(&data)?;

        if address.len() > MAX_ADDR_LEN.into() {
            return Err(AddressError::InvalidLength {
                max: MAX_ADDR_LEN,
                found: address.len(),
            });
        }

        return Ok(Self(address));
    }

    pub fn len(&self) -> u8 {
        self.0
            .len()
            .try_into()
            .expect("MAX_ADDR_LEN is a u8 so this can't fail")
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl Serialize for AccAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AccAddress {
    fn deserialize<D>(deserializer: D) -> Result<AccAddress, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AccAddressVisitor)
    }
}

struct AccAddressVisitor;

impl<'de> serde::de::Visitor<'de> for AccAddressVisitor {
    type Value = AccAddress;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bech32 encoded address")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        AccAddress::from_str(v).map_err(|e| E::custom(format!("invalid address '{}' - {}", v, e)))
    }
}

impl FromStr for AccAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bech32(s)
    }
}

impl TryFrom<Vec<u8>> for AccAddress {
    type Error = AddressError;

    fn try_from(v: Vec<u8>) -> Result<AccAddress, AddressError> {
        if v.len() > MAX_ADDR_LEN.into() {
            return Err(AddressError::InvalidLength {
                max: MAX_ADDR_LEN,
                found: v.len(),
            });
        }
        Ok(AccAddress(v))
    }
}

impl TryFrom<&[u8]> for AccAddress {
    type Error = AddressError;

    fn try_from(v: &[u8]) -> Result<AccAddress, AddressError> {
        if v.len() > MAX_ADDR_LEN.into() {
            return Err(AddressError::InvalidLength {
                max: MAX_ADDR_LEN,
                found: v.len(),
            });
        }
        Ok(AccAddress(v.to_vec()))
    }
}

impl From<AccAddress> for String {
    fn from(v: AccAddress) -> String {
        bech32::encode(ACCOUNT_ADDRESS_PREFIX, v.0.to_base32(), Variant::Bech32)
            .expect("method can only error if HRP is not valid, hard coded HRP is valid")
    }
}

impl Display for AccAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = bech32::encode(ACCOUNT_ADDRESS_PREFIX, self.0.to_base32(), Variant::Bech32)
            .expect("method can only error if HRP is not valid, hard coded HRP is valid");
        write!(f, "{}", addr)
    }
}

impl From<AccAddress> for Vec<u8> {
    fn from(v: AccAddress) -> Vec<u8> {
        v.0
    }
}

#[cfg(test)]
mod tests {

    use bech32::ToBase32;

    use super::*;

    #[test]
    fn from_bech32_success() {
        let input_address = vec![0x00, 0x01, 0x02];
        let encoded = bech32::encode(
            ACCOUNT_ADDRESS_PREFIX,
            input_address.to_base32(),
            Variant::Bech32,
        )
        .unwrap();
        let expected_address = AccAddress(input_address);

        let address = AccAddress::from_bech32(&encoded).unwrap();

        assert_eq!(expected_address, address);
    }

    #[test]
    fn from_bech32_failure_checksum() {
        let input_address = vec![0x00, 0x01, 0x02];
        let mut encoded = bech32::encode(
            ACCOUNT_ADDRESS_PREFIX,
            input_address.to_base32(),
            Variant::Bech32,
        )
        .unwrap();
        encoded.pop();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(err, AddressError::Decode(bech32::Error::InvalidChecksum));
    }

    #[test]
    fn from_bech32_failure_wrong_prefix() {
        let mut hrp = ACCOUNT_ADDRESS_PREFIX.to_string();
        hrp.push_str("atom"); // adding to the ACCOUNT_ADDRESS_PREFIX ensures that hrp is different
        let encoded =
            bech32::encode(&hrp, vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AddressError::InvalidPrefix {
                expected: ACCOUNT_ADDRESS_PREFIX.into(),
                found: hrp,
            }
        );
    }

    #[test]
    fn from_bech32_failure_wrong_variant() {
        let encoded = bech32::encode(
            ACCOUNT_ADDRESS_PREFIX,
            vec![0x00, 0x01, 0x02].to_base32(),
            Variant::Bech32m,
        )
        .unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AddressError::InvalidVariant {
                expected: "Bech32".into(),
                found: "Bech32m".into()
            }
        );
    }

    #[test]
    fn from_bech32_failure_too_long() {
        let encoded = bech32::encode(
            ACCOUNT_ADDRESS_PREFIX,
            vec![0x00; 300].to_base32(),
            Variant::Bech32,
        )
        .unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AddressError::InvalidLength {
                max: 255,
                found: 300
            }
        );
    }

    #[test]
    fn to_string_success() {
        let addr = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string();

        let acc_addr = AccAddress::from_bech32(&addr).unwrap();

        assert_eq!(addr, acc_addr.to_string());
    }

    #[test]
    fn serialize_works() {
        let addr = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string();
        let acc_addr = AccAddress::from_bech32(&addr).unwrap();

        let json = serde_json::to_string(&acc_addr).unwrap();

        assert_eq!(json, r#""cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux""#);
    }

    #[test]
    fn deserialize_works() {
        let json = r#""cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux""#;
        let addr: AccAddress = serde_json::from_str(json).unwrap();
        assert_eq!(
            addr,
            AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux").unwrap()
        )
    }
}
