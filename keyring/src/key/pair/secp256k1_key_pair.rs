use bip32::{DerivationPath, Mnemonic, XPrv};
use k256::ecdsa::signature::Signer;
use k256::ecdsa::SigningKey;
use k256::SecretKey;
use pkcs8::der::pem::PemLabel;
use pkcs8::{
    pkcs5::{pbes2, scrypt},
    rand_core::{OsRng, RngCore},
    DecodePrivateKey, EncodePrivateKey, EncryptedPrivateKeyInfo, LineEnding, PrivateKeyInfo,
};
const HDPATH: &str = "m/44'/118'/0'/0/0";

/// A secp256k1 key pair.
#[derive(Clone, Debug, PartialEq)]
pub struct Secp256k1KeyPair(SecretKey);

impl Secp256k1KeyPair {
    /// Returns PKCS8 PEM encoded private key.
    pub fn to_pkcs8_pem(&self) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        self.0
            .to_pkcs8_pem(LineEnding::default())
            .expect("this can't fail")
    }

    /// Returns PKCS8 PEM encoded private key encrypted with password.
    pub fn to_pkcs8_encrypted_pem(
        &self,
        password: impl AsRef<[u8]>,
    ) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        // TODO: The pkcs8 crate doesn't directly support encrypting with the same scrypt params as openssl.
        // The following implementation is a workaround to achieve the same result.
        // See https://github.com/RustCrypto/formats/issues/1205
        // Once this is fixed, we can replace the following code with:
        // self.secret_key
        //     .to_pkcs8_encrypted_pem(&mut OsRng, password, LineEnding::default())
        //     .expect("this can't fail")

        let mut rng = OsRng;

        let mut salt = [0u8; 16];
        rng.fill_bytes(&mut salt);

        let mut iv = [0u8; 16];
        rng.fill_bytes(&mut iv);

        // 14 = log_2(16384), 32 bytes = 256 bits
        let scrypt_params = scrypt::Params::new(14, 8, 1, 32).unwrap();
        let pbes2_params = pbes2::Parameters::scrypt_aes256cbc(scrypt_params, &salt, &iv).unwrap();

        let plain_text_der = self.0.to_pkcs8_der().unwrap();
        let private_key_info = PrivateKeyInfo::try_from(plain_text_der.as_bytes()).unwrap();

        let secret_doc = private_key_info
            .encrypt_with_params(pbes2_params, password.as_ref())
            .unwrap();

        secret_doc
            .to_pem(EncryptedPrivateKeyInfo::PEM_LABEL, LineEnding::LF)
            .unwrap()
    }

    /// Returns a key pair from a PKCS8 PEM encoded private key.
    pub fn from_pkcs8_pem(s: &str) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self(SecretKey::from_pkcs8_pem(s)?))
    }

    /// Returns a key pair from a PKCS8 PEM encoded private key encrypted with password.
    pub fn from_pkcs8_encrypted_pem(
        s: &str,
        password: impl AsRef<[u8]>,
    ) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self(SecretKey::from_pkcs8_encrypted_pem(s, password)?))
    }

    /// Returns a key pair from a mnemonic.
    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Self {
        let seed = mnemonic.to_seed("");
        let child_path: DerivationPath = HDPATH.parse().expect("hard coded path will never fail");
        let child_xprv = XPrv::derive_from_path(&seed, &child_path)
            .expect("seed has length 64 so this will never return an error");
        let signing_key = child_xprv.private_key();

        Secp256k1KeyPair(signing_key.into())
    }

    /// Signs a message.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signing_key: SigningKey = SigningKey::from(&self.0);
        let signature: k256::ecdsa::Signature = signing_key.sign(message);
        signature.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use pkcs8::der::zeroize::Zeroizing;

    use super::*;

    #[test]
    fn to_pkcs8_pem_works() {
        let expected_pem = "-----BEGIN PRIVATE KEY-----\nMIGEAgEAMBAGByqGSM49AgEGBSuBBAAKBG0wawIBAQQg9v3Q6I45iMwQhpDigYRQ\nhHH0jrooPuth/OhY97epZC+hRANCAAT1BLBR27K+NJ00ploewlmEWRxsH+HKUS7S\nZWkTuFQKKsUHT9nzm6axXiI797T+92b2kfW3JACbcvQ2uTZQWoFE\n-----END PRIVATE KEY-----\n".to_string();
        let expected_pem = Zeroizing::new(expected_pem);
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        let key_pair = Secp256k1KeyPair::from_mnemonic(&mnemonic);

        let pem = key_pair.to_pkcs8_pem();

        assert_eq!(pem, expected_pem);
    }

    #[test]
    fn from_pkcs8_pem_works() {
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        let expected_key_pair = Secp256k1KeyPair::from_mnemonic(&mnemonic);

        let pem_key_pair = Secp256k1KeyPair::from_pkcs8_pem(
            "-----BEGIN PRIVATE KEY-----\nMIGEAgEAMBAGByqGSM49AgEGBSuBBAAKBG0wawIBAQQg9v3Q6I45iMwQhpDigYRQ\nhHH0jrooPuth/OhY97epZC+hRANCAAT1BLBR27K+NJ00ploewlmEWRxsH+HKUS7S\nZWkTuFQKKsUHT9nzm6axXiI797T+92b2kfW3JACbcvQ2uTZQWoFE\n-----END PRIVATE KEY-----\n",
        ).expect("this is a valid PEM");

        assert_eq!(expected_key_pair, pem_key_pair);
    }

    #[test]
    fn encrypted_scenario_works() {
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        let key_pair = Secp256k1KeyPair::from_mnemonic(&mnemonic);

        let pem = key_pair.to_pkcs8_encrypted_pem("password");

        let key_pair_from_pem = Secp256k1KeyPair::from_pkcs8_encrypted_pem(&pem, "password")
            .expect("key pair should be created from pem");

        assert_eq!(key_pair, key_pair_from_pem);
    }

    #[test]
    fn sandpit() {
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        let key_pair = Secp256k1KeyPair::from_mnemonic(&mnemonic);

        let pem = key_pair.to_pkcs8_encrypted_pem("password");

        // write pem string to file
        std::fs::write("./tmp/pem.pem", pem.as_bytes()).unwrap();
    }
}
