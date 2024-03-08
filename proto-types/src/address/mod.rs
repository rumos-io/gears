pub mod prefix;
use std::{
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
};

use bech32::{self, FromBase32, ToBase32, Variant};
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::AddressError;

use self::prefix::{Acc, AddressPrefix, Val};

const MAX_ADDR_LEN: u8 = 255;

pub type AccAddress = BaseAddress<Acc>;
pub type ValAddress = BaseAddress<Val>;

// Note: I am trying avoid bound as much as possible, but here it's better to put one so user always would know that something is wrong
#[derive(Debug, PartialEq, Clone)]
pub struct BaseAddress<PR: AddressPrefix>(Vec<u8>, PhantomData<PR>);

impl<PR: AddressPrefix> BaseAddress<PR> {
    pub fn from_bech32(address: &str) -> Result<Self, AddressError> {
        let (hrp, data, variant) = bech32::decode(address)?;

        if hrp != PR::PREFIX {
            return Err(AddressError::InvalidPrefix {
                expected: PR::PREFIX.to_owned(),
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
        Ok(Self(address, PhantomData))
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

    fn verify_length(v: &[u8]) -> Result<(), AddressError> {
        if v.len() > MAX_ADDR_LEN.into() {
            Err(AddressError::InvalidLength {
                max: MAX_ADDR_LEN,
                found: v.len(),
            })
        } else if v.len() == 0 {
            Err(AddressError::EmptyAddress)
        } else {
            Ok(())
        }
    }

    pub fn as_upper_hex(&self) -> String {
        data_encoding::HEXUPPER.encode(&self.0)
    }
}

impl<PR: AddressPrefix> Serialize for BaseAddress<PR> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, PR: AddressPrefix + serde::de::Visitor<'de>> Deserialize<'de> for BaseAddress<PR> {
    fn deserialize<D>(deserializer: D) -> Result<BaseAddress<PR>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BaseAddressVisitor(PhantomData))
    }
}

struct BaseAddressVisitor<PR: AddressPrefix>(PhantomData<PR>);

impl<'de, PR: AddressPrefix> serde::de::Visitor<'de> for BaseAddressVisitor<PR> {
    type Value = BaseAddress<PR>;

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

impl<PR: AddressPrefix> Display for BaseAddress<PR> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = bech32::encode(PR::PREFIX, self.0.to_base32(), Variant::Bech32)
            .expect("method can only error if HRP is not valid, hard coded HRP is valid");
        write!(f, "{}", addr)
    }
}

impl<PR: AddressPrefix> From<BaseAddress<PR>> for String {
    fn from(v: BaseAddress<PR>) -> String {
        format!("{}", v)
    }
}

impl<PR: AddressPrefix> FromStr for BaseAddress<PR> {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bech32(s)
    }
}

impl<PR: AddressPrefix> TryFrom<Vec<u8>> for BaseAddress<PR> {
    type Error = AddressError;

    fn try_from(v: Vec<u8>) -> Result<BaseAddress<PR>, AddressError> {
        Self::verify_length(&v)?;
        Ok(BaseAddress(v, PhantomData))
    }
}

impl<PR: AddressPrefix> TryFrom<&[u8]> for BaseAddress<PR> {
    type Error = AddressError;

    fn try_from(v: &[u8]) -> Result<BaseAddress<PR>, AddressError> {
        v.to_vec().try_into()
    }
}

impl<PR: AddressPrefix> From<BaseAddress<PR>> for Vec<u8> {
    fn from(v: BaseAddress<PR>) -> Vec<u8> {
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
        let encoded =
            bech32::encode(Acc::PREFIX, input_address.to_base32(), Variant::Bech32).unwrap();
        let expected_address = BaseAddress::<Acc>(input_address, PhantomData);

        let address = AccAddress::from_bech32(&encoded).unwrap();

        assert_eq!(expected_address, address);
    }

    #[test]
    fn from_bech32_failure_checksum() {
        let input_address = vec![0x00, 0x01, 0x02];
        let mut encoded =
            bech32::encode(Acc::PREFIX, input_address.to_base32(), Variant::Bech32).unwrap();
        encoded.pop();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(err, AddressError::Decode(bech32::Error::InvalidChecksum));
    }

    #[test]
    fn from_bech32_failure_wrong_prefix() {
        let mut hrp = Acc::PREFIX.to_string();
        hrp.push_str("atom"); // adding to the BECH_32_PREFIX_ACC_ADDR ensures that hrp is different
        let encoded =
            bech32::encode(&hrp, vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AddressError::InvalidPrefix {
                expected: Acc::PREFIX.into(),
                found: hrp,
            }
        );
    }

    #[test]
    fn from_bech32_failure_wrong_variant() {
        let encoded = bech32::encode(
            Acc::PREFIX,
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
        let encoded =
            bech32::encode(Acc::PREFIX, vec![0x00; 300].to_base32(), Variant::Bech32).unwrap();

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
        let encoded = bech32::encode(Acc::PREFIX, vec![].to_base32(), Variant::Bech32).unwrap();

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
