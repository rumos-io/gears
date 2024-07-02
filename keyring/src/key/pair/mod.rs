pub mod secp256k1_key_pair;

use serde::{Deserialize, Serialize};

use self::secp256k1_key_pair::Secp256k1KeyPair;

/// A key pair.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "secret_key")]
pub enum KeyPair {
    #[serde(rename = "secp256k1")]
    #[serde(with = "hex::serde")]
    Secp256k1(Secp256k1KeyPair),
}

impl KeyPair {
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

    /// Signs a message.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        match self {
            KeyPair::Secp256k1(key) => key.sign(message).into(),
        }
    }

    pub fn from_mnemonic(mnemonic: &bip32::Mnemonic) -> Self {
        Self::Secp256k1(Secp256k1KeyPair::from_mnemonic(mnemonic))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip32::Mnemonic;

    #[test]
    fn test_key_pair_serialization() {
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        let key_pair = KeyPair::Secp256k1(Secp256k1KeyPair::from_mnemonic(&mnemonic));

        let serialized = serde_json::to_string(&key_pair).unwrap();

        assert_eq!(
            serialized,
            r#"{"type":"secp256k1","secret_key":"f6fdd0e88e3988cc108690e28184508471f48eba283eeb61fce858f7b7a9642f"}"#
        );
    }
}
