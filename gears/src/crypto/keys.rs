use std::{convert::Infallible, error::Error};

use bip32::PublicKey as PublicKeyTrait;
use keyring::key::pair::KeyPair;

use crate::types::address::AccAddress;

use super::{public::PublicKey, secp256k1::Secp256k1PubKey};

pub const SIZE_ERR_MSG: &str =
    "ripemd160 digest size is 160 bytes which is less than AccAddress::MAX_ADDR_LEN";

pub trait GearsPublicKey {
    /// Returns a Gears public key.
    fn get_gears_public_key(&self) -> PublicKey;
}

pub trait ReadAccAddress {
    /// Returns a Bitcoin style addresses: RIPEMD160(SHA256(pubkey)).
    fn get_address(&self) -> AccAddress;
}

pub trait SigningKey {
    type Error: Error;
    /// Signs the given message.
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

impl GearsPublicKey for KeyPair {
    fn get_gears_public_key(&self) -> PublicKey {
        match self {
            KeyPair::Secp256k1(key) => {
                let raw_public_key = key.inner().public_key().to_bytes().to_vec();
                let public_key: Secp256k1PubKey = raw_public_key.try_into().expect(
                    "raw public key is a valid secp256k1 public key so this will always succeed",
                );
                PublicKey::Secp256k1(public_key)
            }
        }
    }
}

impl ReadAccAddress for KeyPair {
    fn get_address(&self) -> AccAddress {
        self.get_gears_public_key().get_address()
    }
}

impl SigningKey for KeyPair {
    type Error = Infallible;

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(self.sign(message))
    }
}
