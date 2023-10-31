use bip32::{DerivationPath, Mnemonic, PublicKey, XPrv};
use k256::SecretKey;
use pkcs8::{rand_core::OsRng, DecodePrivateKey, EncodePrivateKey, LineEnding};
use proto_types::AccAddress;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

const HDPATH: &str = "m/44'/118'/0'/0/0";

#[derive(Clone, Debug, PartialEq)]
pub struct Secp256k1KeyPair {
    secret_key: SecretKey,
}

impl Secp256k1KeyPair {
    /// Returns PKCS8 PEM encoded private key
    pub fn to_pkcs8_pem(&self) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        self.secret_key
            .to_pkcs8_pem(LineEnding::default())
            .expect("this can't fail")
    }

    /// Returns PKCS8 PEM encoded private key encrypted with password
    pub fn to_pkcs8_encrypted_pem(
        &self,
        password: impl AsRef<[u8]>,
    ) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        self.secret_key
            .to_pkcs8_encrypted_pem(&mut OsRng, password, LineEnding::default())
            .expect("this can't fail")
    }

    /// Returns a key pair from a PKCS8 PEM encoded private key
    pub fn from_pkcs8_pem(s: &str) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self {
            secret_key: SecretKey::from_pkcs8_pem(s)?,
        })
    }

    /// Returns a key pair from a PKCS8 PEM encoded private key encrypted with password
    pub fn from_pkcs8_encrypted_pem(
        s: &str,
        password: impl AsRef<[u8]>,
    ) -> Result<Self, k256::pkcs8::Error> {
        Ok(Self {
            secret_key: SecretKey::from_pkcs8_encrypted_pem(s, password)?,
        })
    }

    /// Returns a key pair from a mnemonic
    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Self {
        let seed = mnemonic.to_seed("");
        let child_path: DerivationPath = HDPATH.parse().expect("hard coded path will never fail");
        let child_xprv = XPrv::derive_from_path(&seed, &child_path)
            .expect("seed has length 64 so this will never return an error");
        let signing_key = child_xprv.private_key();

        Secp256k1KeyPair {
            secret_key: signing_key.into(),
        }
    }

    /// Returns a Bitcoin style addresses: RIPEMD160(SHA256(pubkey))
    pub fn get_address(&self) -> AccAddress {
        let pub_key = self.secret_key.public_key().to_bytes().to_vec();

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
}
