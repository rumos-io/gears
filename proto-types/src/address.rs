use std::fmt::{self, Display};

use bech32::{self, FromBase32, ToBase32, Variant};

use crate::error::AddressError;

#[derive(Debug, PartialEq, Clone)]
pub struct AccAddress(Vec<u8>);

const MAX_ADDR_LEN: u8 = 255;
const BECH32_MAIN_PREFIX: &str = "cosmos";
const BECH32_PREFIX_ACC_ADDR: &str = BECH32_MAIN_PREFIX;

impl AccAddress {
    pub fn from_bech32(address: &str) -> Result<Self, AddressError> {
        let (hrp, data, variant) = bech32::decode(address)?;

        if hrp != BECH32_PREFIX_ACC_ADDR {
            return Err(AddressError::InvalidPrefix {
                expected: BECH32_PREFIX_ACC_ADDR.into(),
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
        // returns a Result there's no harm in returning an error here.
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
        self.0.len().try_into().expect("MAX_ADDR_LEN is also a u8")
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

impl From<AccAddress> for String {
    fn from(v: AccAddress) -> String {
        bech32::encode(BECH32_PREFIX_ACC_ADDR, v.0.to_base32(), Variant::Bech32)
            .expect("can only error if HRP is not valid, which can never happen")
    }
}

impl Display for AccAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = bech32::encode(BECH32_PREFIX_ACC_ADDR, self.0.to_base32(), Variant::Bech32)
            .expect("can only error if HRP is not valid, which can never happen");
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
        let encoded = bech32::encode("cosmos", input_address.to_base32(), Variant::Bech32).unwrap();
        let expected_address = AccAddress(input_address);

        let address = AccAddress::from_bech32(&encoded).unwrap();

        assert_eq!(expected_address, address);
    }

    #[test]
    fn from_bech32_failure_checksum() {
        let input_address = vec![0x00, 0x01, 0x02];
        let mut encoded =
            bech32::encode("cosmos", input_address.to_base32(), Variant::Bech32).unwrap();
        encoded.pop();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(err, AddressError::Decode(bech32::Error::InvalidChecksum));
    }

    #[test]
    fn from_bech32_failure_wrong_prefix() {
        let encoded =
            bech32::encode("atom", vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AddressError::InvalidPrefix {
                expected: "cosmos".into(),
                found: "atom".into()
            }
        );
    }

    #[test]
    fn from_bech32_failure_wrong_variant() {
        let encoded = bech32::encode(
            "cosmos",
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
            bech32::encode("cosmos", vec![0x00; 300].to_base32(), Variant::Bech32).unwrap();

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
}
