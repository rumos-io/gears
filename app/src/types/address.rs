use crate::error::AppError;
use bech32::{self, FromBase32, ToBase32, Variant};

//TODO: finish testing

#[derive(Debug, PartialEq, Clone)]
pub struct AccAddress(Vec<u8>);

const MAX_ADDR_LEN: u8 = 255;
const BECH32_MAIN_PREFIX: &str = "cosmos";
const BECH32_PREFIX_ACC_ADDR: &str = BECH32_MAIN_PREFIX;

impl AccAddress {
    pub fn from_bech32(address: &str) -> Result<Self, AppError> {
        let (hrp, data, variant) = bech32::decode(address)?;

        if hrp != BECH32_PREFIX_ACC_ADDR {
            return Err(AppError::InvalidAddress(
                format!(
                    "Address has wrong prefix, expected {}, found {}",
                    BECH32_PREFIX_ACC_ADDR, hrp
                )
                .into(),
            ));
        };

        if let Variant::Bech32m = variant {
            return Err(AppError::InvalidAddress(
                "Incorrect variant, expected Bech32, found Bech32m".into(),
            ));
        }

        // It's unclear whether the conversion from base32 can ever fail. Since this method
        // returns a Result there's no harm in returning an error here.
        let address = Vec::<u8>::from_base32(&data)?;

        if address.len() > MAX_ADDR_LEN.into() {
            return Err(AppError::InvalidAddress(
                format!("Decoded address has length greater than {}", MAX_ADDR_LEN).into(),
            ));
        }

        return Ok(Self(address));
    }

    pub fn len(&self) -> u8 {
        self.0.len().try_into().expect("MAX_ADDR_LEN is also a u8")
    }
}

impl TryFrom<Vec<u8>> for AccAddress {
    type Error = AppError;

    fn try_from(v: Vec<u8>) -> Result<AccAddress, AppError> {
        if v.len() > MAX_ADDR_LEN.into() {
            return Err(AppError::InvalidAddress(
                format!("Decoded address has length greater than {}", MAX_ADDR_LEN).into(),
            ));
        }
        Ok(AccAddress(v))
    }
}

impl From<AccAddress> for String {
    fn from(v: AccAddress) -> String {
        bech32::encode(BECH32_PREFIX_ACC_ADDR, v.0.to_base32(), Variant::Bech32)
            .expect("AccAddress should contain valid bech32 address")
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

        assert_eq!(err, AppError::Bech32(bech32::Error::InvalidChecksum));
    }

    #[test]
    fn from_bech32_failure_wrong_prefix() {
        let encoded =
            bech32::encode("atom", vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AppError::InvalidAddress(
                "Address has wrong prefix, expected cosmos, found atom".into()
            )
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
            AppError::InvalidAddress("Incorrect variant, expected Bech32, found Bech32m".into())
        );
    }

    #[test]
    fn from_bech32_failure_too_long() {
        let encoded =
            bech32::encode("cosmos", vec![0x00; 300].to_base32(), Variant::Bech32).unwrap();

        println!("{}", encoded);

        let err = AccAddress::from_bech32(&encoded).unwrap_err();

        assert_eq!(
            err,
            AppError::InvalidAddress("Decoded address has length greater than 255".into())
        );
    }
}
