use std::path::PathBuf;

use crate::{
    error::Error,
    key_pair::{secp256k1_key_pair::Secp256k1KeyPair, KeyPair},
    key_store::FileStore,
};
use bip32::Mnemonic;
use k256::elliptic_curve::rand_core;

use rand_core::OsRng;

/// Used to specify the type of key to generate.
pub enum KeyType {
    Secp256k1,
}

#[derive(Debug)]
enum KeyStore {
    File(FileStore),
    Test(FileStore),
}

/// A keyring. Used to store and retrieve keys.
#[derive(Debug)]
pub struct Keyring {
    store: KeyStore,
}

impl Keyring {
    /// Opens a keyring from a directory, using the password provided.
    /// If the keyring does not exist, it will be created if create is true.
    pub fn open_file(path: PathBuf, password: String, create: bool) -> Result<Self, Error> {
        Ok(Self {
            store: KeyStore::File(FileStore::open(path, Some(password), create)?),
        })
    }

    /// Opens a test keyring from a directory. The keys are not encrypted, so this
    /// should only be used for testing. If the keyring does not exist, it will be
    /// created if create is true.
    pub fn open_test(path: PathBuf, create: bool) -> Result<Self, Error> {
        Ok(Self {
            store: KeyStore::Test(FileStore::open(path, None, create)?),
        })
    }

    /// Generates a key pair from the mnemonic provided and stores the keypair.
    pub fn add_key<S>(&self, name: S, mnemonic: &Mnemonic, key_type: KeyType) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        let key_pair = match key_type {
            KeyType::Secp256k1 => KeyPair::Secp256k1(Secp256k1KeyPair::from_mnemonic(mnemonic)),
        };

        match &self.store {
            KeyStore::File(file_store) => file_store.set_key_pair(name, key_pair),
            KeyStore::Test(file_store) => file_store.set_key_pair(name, key_pair),
        }
    }

    /// Generates a new random mnemonic and key pair, stores the new key pair and
    /// returns the generated mnemonic.
    pub fn create_key<S>(&self, name: S, key_type: KeyType) -> Result<Mnemonic, Error>
    where
        S: AsRef<str>,
    {
        let mnemonic = Mnemonic::random(&mut OsRng, bip32::Language::English);
        self.add_key(name, &mnemonic, key_type)?;
        Ok(mnemonic)
    }

    /// Get a key by name.
    pub fn get_key_by_name<S>(&self, name: S) -> Result<KeyPair, Error>
    where
        S: AsRef<str>,
    {
        //TODO: return key wrapped in Secret
        match &self.store {
            KeyStore::File(file_store) => file_store.get_key_by_name(name),
            KeyStore::Test(file_store) => file_store.get_key_by_name(name),
        }
    }

    /// Delete a key by name.
    pub fn delete_key_by_name<S>(&self, name: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        match &self.store {
            KeyStore::File(file_store) => file_store.delete_key_by_name(name),
            KeyStore::Test(file_store) => file_store.delete_key_by_name(name),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn keyring_test_scenario_works() {
        let path = PathBuf::from("./tmp/keyring_test_scenario_works");
        let _ = std::fs::remove_dir_all(&path);
        let key_ring = Keyring::open_test(path.clone(), true).expect("keyring should open");

        // add key should succeed
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        key_ring
            .add_key("bob", &mnemonic, KeyType::Secp256k1)
            .expect("key should be added");

        // add key with same name should fail
        let error = key_ring
            .add_key("bob", &mnemonic, KeyType::Secp256k1)
            .expect_err("key should not be added");

        assert!(matches!(error, Error::AlreadyExists { .. }));

        // get key should succeed
        key_ring
            .get_key_by_name("bob")
            .expect("key should be retrieved");

        // delete key should succeed
        key_ring
            .delete_key_by_name("bob")
            .expect("key should be deleted");

        // get key should fail
        let error = key_ring
            .get_key_by_name("bob")
            .expect_err("key should not be retrieved");

        assert!(matches!(error, Error::DoesNotExist { .. }));

        // delete key should fail
        let error = key_ring
            .delete_key_by_name("bob")
            .expect_err("key should not be deleted");

        assert!(matches!(error, Error::DoesNotExist { .. }));

        // create key should succeed
        key_ring
            .create_key("bob", KeyType::Secp256k1)
            .expect("key should be created");

        // get key should succeed
        key_ring
            .get_key_by_name("bob")
            .expect("key should be retrieved");

        std::fs::remove_dir_all(path.clone()).expect("tmp directory should be deleted");

        // open keyring should fail
        let error =
            Keyring::open_test(path.clone(), false).expect_err("keyring should fail to open");

        assert!(matches!(error, Error::KeyringDoesNotExist(_)));
    }

    #[test]
    fn keyring_file_scenario_works() {
        let path = PathBuf::from("./tmp/keyring_file_scenario_works");
        let _ = std::fs::remove_dir_all(&path);

        let key_ring =
            Keyring::open_file(path.clone(), "test".into(), true).expect("keyring should open");

        // add key should succeed
        let mnemonic = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English).unwrap();
        key_ring
            .add_key("bob", &mnemonic, KeyType::Secp256k1)
            .expect("key should be added");

        // re open keyring with same password should succeed
        let key_ring =
            Keyring::open_file(path.clone(), "test".into(), false).expect("keyring should open");

        // get key should succeed
        key_ring
            .get_key_by_name("bob")
            .expect("key should be retrieved");

        // re open keyring with wrong password should fail
        let error = Keyring::open_file(path, "wrong".into(), false)
            .expect_err("keyring should fail to open");

        assert!(matches!(error, Error::IncorrectPassword));
    }
}
