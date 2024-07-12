use bech32::{FromBase32, ToBase32, Variant};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

const PREFIX_VALIDATOR: &str = "val";
const PREFIX_OPERATOR: &str = "oper";
const PREFIX_CONSENSUS: &str = "cons";

const BECH_32_PREFIX_ACC_ADDR: &str = env!("BECH_32_MAIN_PREFIX");
const BECH_32_PREFIX_VAL_ADDR: &str = constcat::concat!(
    env!("BECH_32_MAIN_PREFIX"),
    PREFIX_VALIDATOR,
    PREFIX_OPERATOR
);
const BECH_32_PREFIX_CONS_ADDR: &str = constcat::concat!(
    env!("BECH_32_MAIN_PREFIX"),
    PREFIX_VALIDATOR,
    PREFIX_CONSENSUS
);

const MAX_ADDR_LEN: u8 = 255;

pub type AccAddress = BaseAddress<0>;
pub type ValAddress = BaseAddress<1>;
pub type ConsAddress = BaseAddress<2>;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct BaseAddress<const PREFIX: u8>(Vec<u8>);

impl<const PREFIX: u8> BaseAddress<PREFIX> {
    pub fn from_bech32(address: &str) -> Result<Self, AddressError> {
        let (hrp, data, variant) = bech32::decode(address)?;

        let prefix = match PREFIX {
            0 => BECH_32_PREFIX_ACC_ADDR,
            1 => BECH_32_PREFIX_VAL_ADDR,
            _ => BECH_32_PREFIX_CONS_ADDR,
        };

        if hrp != prefix {
            return Err(AddressError::InvalidPrefix {
                expected: prefix.into(),
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

        Self::verify_length(&address)?;
        Ok(Self(address))
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u8 {
        self.0
            .len()
            .try_into()
            .expect("MAX_ADDR_LEN is a u8 so this can't fail")
    }

    pub fn as_hex(&self) -> String {
        data_encoding::HEXLOWER.encode(&self.0)
    }

    fn verify_length(v: &[u8]) -> Result<(), AddressError> {
        if v.len() > MAX_ADDR_LEN.into() {
            Err(AddressError::InvalidLength {
                max: MAX_ADDR_LEN,
                found: v.len(),
            })
        } else if v.is_empty() {
            Err(AddressError::EmptyAddress)
        } else {
            Ok(())
        }
    }

    pub fn as_upper_hex(&self) -> String {
        data_encoding::HEXUPPER.encode(&self.0)
    }
}

impl<const PREFIX: u8> AsRef<[u8]> for BaseAddress<PREFIX> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const PREFIX: u8> Serialize for BaseAddress<PREFIX> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, const PREFIX: u8> Deserialize<'de> for BaseAddress<PREFIX> {
    fn deserialize<D>(deserializer: D) -> Result<BaseAddress<PREFIX>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BaseAddressVisitor)
    }
}

struct BaseAddressVisitor<const PREFIX: u8>;

impl<'de, const PREFIX: u8> serde::de::Visitor<'de> for BaseAddressVisitor<PREFIX> {
    type Value = BaseAddress<PREFIX>;

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

impl<const PREFIX: u8> Display for BaseAddress<PREFIX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hrp = match PREFIX {
            0 => BECH_32_PREFIX_ACC_ADDR,
            1 => BECH_32_PREFIX_VAL_ADDR,
            _ => BECH_32_PREFIX_CONS_ADDR,
        };

        let addr = bech32::encode(hrp, self.0.to_base32(), Variant::Bech32)
            .expect("method can only error if HRP is not valid, hard coded HRP is valid");
        write!(f, "{}", addr)
    }
}

impl<const PREFIX: u8> From<BaseAddress<PREFIX>> for String {
    fn from(v: BaseAddress<PREFIX>) -> String {
        format!("{}", v)
    }
}

impl<const PREFIX: u8> FromStr for BaseAddress<PREFIX> {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bech32(s)
    }
}

impl<const PREFIX: u8> TryFrom<Vec<u8>> for BaseAddress<PREFIX> {
    type Error = AddressError;

    fn try_from(v: Vec<u8>) -> Result<BaseAddress<PREFIX>, AddressError> {
        Self::verify_length(&v)?;
        Ok(BaseAddress(v))
    }
}

impl<const PREFIX: u8> TryFrom<&[u8]> for BaseAddress<PREFIX> {
    type Error = AddressError;

    fn try_from(v: &[u8]) -> Result<BaseAddress<PREFIX>, AddressError> {
        v.to_vec().try_into()
    }
}

impl<const PREFIX: u8> From<BaseAddress<PREFIX>> for Vec<u8> {
    fn from(v: BaseAddress<PREFIX>) -> Vec<u8> {
        v.0
    }
}

// TODO: CHECK IS IT SAFE TO CONVERT ONE KEY TO OTHER
impl From<AccAddress> for ValAddress {
    fn from(value: AccAddress) -> Self {
        Self(value.0)
    }
}
impl From<ValAddress> for ConsAddress {
    fn from(value: ValAddress) -> Self {
        Self(value.0)
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    #[error(transparent)]
    Decode(#[from] bech32::Error),

    #[error("address has wrong prefix (expected {expected:?}, found {found:?})")]
    InvalidPrefix { expected: String, found: String },

    #[error("invalid variant (expected {expected:?}, found {found:?})")]
    InvalidVariant { expected: String, found: String },

    #[error("invalid length, max length is: {max:?}, found {found:?})")]
    InvalidLength { max: u8, found: usize },

    #[error("bech32 decode error: address is empty")]
    EmptyAddress,
}

#[cfg(test)]
mod tests {

    use bech32::ToBase32;

    use super::*;

    #[test]
    fn from_bech32_success() {
        let input_address = vec![0x00, 0x01, 0x02];
        let encoded = bech32::encode(
            BECH_32_PREFIX_ACC_ADDR,
            input_address.to_base32(),
            Variant::Bech32,
        )
        .unwrap();
        let expected_address = BaseAddress::<0>(input_address);

        let address = AccAddress::from_bech32(&encoded).unwrap();

        assert_eq!(expected_address, address);
    }

    #[test]
    fn from_bech32_failure_checksum() {
        let input_address = vec![0x00, 0x01, 0x02];
        let mut encoded = bech32::encode(
            BECH_32_PREFIX_ACC_ADDR,
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
        let mut hrp = BECH_32_PREFIX_ACC_ADDR.to_string();
        hrp.push_str("atom"); // adding to the BECH_32_PREFIX_ACC_ADDR ensures that hrp is different
        let encoded =
            bech32::encode(&hrp, vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AddressError::InvalidPrefix {
                expected: BECH_32_PREFIX_ACC_ADDR.into(),
                found: hrp,
            }
        );
    }

    #[test]
    fn from_bech32_failure_wrong_variant() {
        let encoded = bech32::encode(
            BECH_32_PREFIX_ACC_ADDR,
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
            BECH_32_PREFIX_ACC_ADDR,
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
    fn from_bech32_failure_empty_address() {
        let encoded =
            bech32::encode(BECH_32_PREFIX_ACC_ADDR, vec![].to_base32(), Variant::Bech32).unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(err, AddressError::EmptyAddress);
    }

    #[test]
    fn from_slice_failure_empty_address() {
        let address: Vec<u8> = vec![];
        let err = AccAddress::try_from(address.as_slice()).unwrap_err();
        assert_eq!(err, AddressError::EmptyAddress);
    }

    #[test]
    fn to_string_success() {
        let addr = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string();

        let acc_addr = AccAddress::from_bech32(&addr).unwrap();

        assert_eq!(addr, acc_addr.to_string());
    }

    #[test]
    fn string_from_self_success() {
        let addr = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string();

        let acc_addr = AccAddress::from_bech32(&addr).unwrap();

        assert_eq!(addr, String::from(acc_addr));
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
