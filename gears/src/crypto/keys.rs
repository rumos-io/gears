use bip32::PublicKey as PublicKeyTrait;
use ibc_types::address::AccAddress;
use keyring::key::{
    pair::{secp256k1_key_pair::Secp256k1KeyPair, KeyPair},
    public::PublicKey,
    secp256k1::Secp256k1PubKey,
};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

const SIZE_ERR_MSG: &str =
    "ripemd160 digest size is 160 bytes which is less than AccAddress::MAX_ADDR_LEN";

pub trait GearsPublicKey {
    /// Returns a Gears public key.
    fn get_gears_public_key(&self) -> PublicKey;
}

pub trait ReadAccAddress {
    /// Returns a Bitcoin style addresses: RIPEMD160(SHA256(pubkey)).
    fn get_address(&self) -> AccAddress;
}

impl GearsPublicKey for Secp256k1KeyPair {
    fn get_gears_public_key(&self) -> PublicKey {
        let raw_public_key = self.inner().public_key().to_bytes().to_vec();
        let public_key: Secp256k1PubKey = raw_public_key
            .try_into()
            .expect("raw public key is a valid secp256k1 public key so this will always succeed");

        PublicKey::Secp256k1(public_key)
    }
}

impl GearsPublicKey for KeyPair {
    fn get_gears_public_key(&self) -> PublicKey {
        match self {
            KeyPair::Secp256k1(key) => key.get_gears_public_key(),
        }
    }
}

impl ReadAccAddress for KeyPair {
    fn get_address(&self) -> AccAddress {
        match self {
            KeyPair::Secp256k1(key) => key.get_address(),
        }
    }
}

impl ReadAccAddress for Secp256k1KeyPair {
    fn get_address(&self) -> AccAddress {
        let pub_key = self.inner().public_key().to_bytes().to_vec();

        // sha256 hash
        let mut hasher = Sha256::new();
        hasher.update(&pub_key);
        let hash = hasher.finalize();

        // ripemd160 hash
        let mut hasher = Ripemd160::new();
        hasher.update(hash);
        let hash = hasher.finalize();

        hash.as_slice().try_into().expect(SIZE_ERR_MSG)
    }
}

impl ReadAccAddress for Secp256k1PubKey {
    fn get_address(&self) -> AccAddress {
        let mut hasher = Sha256::new();
        hasher.update(&Vec::from(self.to_owned()));
        let hash = hasher.finalize();

        let mut hasher = Ripemd160::new();
        hasher.update(hash);
        let hash = hasher.finalize();

        let res: AccAddress = hash.as_slice().try_into().expect(SIZE_ERR_MSG);

        res
    }
}

impl ReadAccAddress for PublicKey {
    fn get_address(&self) -> AccAddress {
        match self {
            PublicKey::Secp256k1(key) => key.get_address(),
        }
    }
}
