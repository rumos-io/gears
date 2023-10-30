use bip32::{DerivationPath, Mnemonic, PublicKey, XPrv};
use k256::SecretKey;
use pkcs8::{rand_core::OsRng, DecodePrivateKey, EncodePrivateKey, LineEnding};
use proto_types::AccAddress;

#[derive(Clone, Debug, PartialEq)]
pub struct Secp256k1KeyPair {
    pub address: AccAddress,
    secret_key: SecretKey,
}

impl Secp256k1KeyPair {
    pub fn to_pkcs8_pem(&self) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        self.secret_key
            .to_pkcs8_pem(LineEnding::default())
            .expect("this can't fail")
    }

    pub fn to_pkcs8_encrypted_pem(
        &self,
        password: impl AsRef<[u8]>,
    ) -> k256::elliptic_curve::zeroize::Zeroizing<String> {
        self.secret_key
            .to_pkcs8_encrypted_pem(&mut OsRng, password, LineEnding::default())
            .expect("this can't fail")
    }

    pub fn from_pkcs8_pem(s: &str) -> Result<Self, k256::pkcs8::Error> {
        let secret_key = SecretKey::from_pkcs8_pem(s)?;

        let public_key = secret_key.public_key().to_bytes();

        let address = proto_messages::cosmos::crypto::secp256k1::v1beta1::PubKey::try_from(
            public_key.to_vec(),
        )
        .unwrap() // TODO: need a more direct approach
        .get_address();

        Ok(Self {
            address,
            secret_key,
        })
    }

    pub fn from_pkcs8_encrypted_pem(
        s: &str,
        password: impl AsRef<[u8]>,
    ) -> Result<Self, k256::pkcs8::Error> {
        let secret_key = SecretKey::from_pkcs8_encrypted_pem(s, password)?;

        let public_key = secret_key.public_key().to_bytes();

        let address = proto_messages::cosmos::crypto::secp256k1::v1beta1::PubKey::try_from(
            public_key.to_vec(),
        )
        .unwrap() // TODO: need a more direct approach
        .get_address();

        Ok(Self {
            address,
            secret_key,
        })
    }

    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Self {
        let seed = mnemonic.to_seed("");

        let child_path: DerivationPath = "m/44'/118'/0'/0/0"
            .parse()
            .expect("hard coded path will never fail");
        let child_xprv = XPrv::derive_from_path(&seed, &child_path)
            .expect("seed has length 64 so this will never return an error");

        let child_xpub = child_xprv.public_key();
        let signing_key = child_xprv.private_key();
        let key: SecretKey = signing_key.into();
        let pub_key = child_xpub.public_key().to_bytes();

        let address =
            proto_messages::cosmos::crypto::secp256k1::v1beta1::PubKey::try_from(pub_key.to_vec())
                .unwrap() // TODO: need a more direct approach
                .get_address();

        Secp256k1KeyPair {
            address,
            secret_key: key,
        }
    }
}

// write tests
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
}
