use bip32::{DerivationPath, Mnemonic, PublicKey as PublicKeyBid32, XPrv};
use k256::ecdsa::signature::Signer;
use k256::ecdsa::SigningKey;
use k256::SecretKey;
use pkcs8::der::pem::PemLabel;
use pkcs8::{
    pkcs5::{pbes2, scrypt},
    rand_core::{OsRng, RngCore},
    DecodePrivateKey, EncodePrivateKey, EncryptedPrivateKeyInfo, LineEnding, PrivateKeyInfo,
};
// use proto_messages::cosmos::crypto::secp256k1::v1beta1::PubKey;
// use proto_messages::cosmos::tx::v1beta1::tx::public_key::PublicKey as GearsPublicKey;
use proto_types::AccAddress;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use crate::crypto::secp256k1::Secp256k1PubKey;
use crate::public_key::PublicKey;

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

    /// Returns a Bitcoin style addresses: RIPEMD160(SHA256(pubkey)).
    pub fn get_address(&self) -> AccAddress {
        let pub_key = self.0.public_key().to_bytes().to_vec();

        // sha256 hash
        let mut hasher = Sha256::new();
        hasher.update(&pub_key);
        let hash = hasher.finalize();

        // ripemd160 hash
        let mut hasher = Ripemd160::new();
        hasher.update(hash);
        let hash = hasher.finalize();

        hash.as_slice().try_into().expect(
            "ripemd160 digest size is 160 bytes which is less than AccAddress::MAX_ADDR_LEN",
        )
    }

    /// Returns a Gears public key.
    pub fn get_gears_public_key(&self) -> PublicKey {
        let raw_public_key = self.0.public_key().to_bytes().to_vec();
        let public_key: Secp256k1PubKey = raw_public_key
            .try_into()
            .expect("raw public key is a valid secp256k1 public key so this will always succeed");

        PublicKey::Secp256k1(public_key)
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
    fn from_mnemonic_and_get_address_works() {
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        let key_pair = Secp256k1KeyPair::from_mnemonic(&mnemonic);

        assert!(matches!(
            key_pair.get_address().to_string().as_str(),
            "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux"
        ));
    }

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
