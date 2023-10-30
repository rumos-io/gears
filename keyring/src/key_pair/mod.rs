pub mod secp256k1_key_pair;

use proto_types::AccAddress;

use self::secp256k1_key_pair::Secp256k1KeyPair;

#[derive(Clone, Debug)]
pub enum KeyPair {
    Secp256k1(Secp256k1KeyPair),
}

impl KeyPair {
    pub fn get_address(&self) -> &AccAddress {
        match self {
            KeyPair::Secp256k1(key) => &key.address,
        }
    }

    pub fn to_pkcs8_pem(&self) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        match self {
            KeyPair::Secp256k1(key) => key.to_pkcs8_pem(),
        }
    }

    pub fn to_pkcs8_encrypted_pem(
        &self,
        password: impl AsRef<[u8]>,
    ) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        match self {
            KeyPair::Secp256k1(key) => key.to_pkcs8_encrypted_pem(password),
        }
    }

    pub fn from_pkcs8_pem(s: &str) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self::Secp256k1(Secp256k1KeyPair::from_pkcs8_pem(s)?))
    }

    pub fn from_pkcs8_encrypted_pem(
        s: &str,
        password: impl AsRef<[u8]>,
    ) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self::Secp256k1(Secp256k1KeyPair::from_pkcs8_encrypted_pem(
            s, password,
        )?))
    }
}
