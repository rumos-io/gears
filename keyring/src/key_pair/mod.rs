pub mod secp256k1_key_pair;

use proto_messages::cosmos::tx::v1beta1::PublicKey;
use proto_types::AccAddress;

use self::secp256k1_key_pair::Secp256k1KeyPair;

/// A key pair.
#[derive(Clone, Debug)]
pub enum KeyPair {
    Secp256k1(Secp256k1KeyPair),
}

impl KeyPair {
    /// Generates a key pair from the mnemonic provided and stores the keypair.
    pub fn get_address(&self) -> AccAddress {
        match self {
            KeyPair::Secp256k1(key) => key.get_address(),
        }
    }

    /// Returns PKCS8 PEM encoded private key.
    pub fn to_pkcs8_pem(&self) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        match self {
            KeyPair::Secp256k1(key) => key.to_pkcs8_pem(),
        }
    }

    /// Returns PKCS8 PEM encoded private key encrypted with password.
    pub fn to_pkcs8_encrypted_pem(
        &self,
        password: impl AsRef<[u8]>,
    ) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        match self {
            KeyPair::Secp256k1(key) => key.to_pkcs8_encrypted_pem(password),
        }
    }

    /// Returns a key pair from a PKCS8 PEM encoded private key.
    pub fn from_pkcs8_pem(s: &str) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self::Secp256k1(Secp256k1KeyPair::from_pkcs8_pem(s)?))
    }

    /// Returns a key pair from a PKCS8 PEM encoded private key encrypted with password.
    pub fn from_pkcs8_encrypted_pem(
        s: &str,
        password: impl AsRef<[u8]>,
    ) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self::Secp256k1(Secp256k1KeyPair::from_pkcs8_encrypted_pem(
            s, password,
        )?))
    }

    ///Returns a gears public key
    pub fn get_gears_public_key(&self) -> PublicKey {
        match self {
            KeyPair::Secp256k1(key) => key.get_gears_public_key(),
        }
    }

    /// Signs a message.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        match self {
            KeyPair::Secp256k1(key) => key.sign(message),
        }
    }
}
