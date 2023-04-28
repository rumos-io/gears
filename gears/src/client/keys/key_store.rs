use anyhow::{anyhow, Result};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::marker::PhantomData;
use std::path::PathBuf;

use ibc_relayer::keyring::SigningKeyPairSized;
use serde::{Deserialize, Serialize};

pub const KEYSTORE_DISK_BACKEND: &str = "keyring-test";
pub const KEYSTORE_FILE_EXTENSION: &str = "json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskStore<S> {
    path: PathBuf,
    phantom: PhantomData<S>,
}

impl<S> DiskStore<S> {
    pub fn new(mut home: PathBuf) -> Result<Self> {
        home.push(KEYSTORE_DISK_BACKEND);

        // Create keys folder if it does not exist
        fs::create_dir_all(&home)?;

        Ok(Self {
            path: home,
            phantom: PhantomData,
        })
    }
}

impl<S: SigningKeyPairSized> DiskStore<S> {
    pub fn get_key(&self, key_name: &str) -> Result<S> {
        let mut key_file = self.path.join(key_name);
        key_file.set_extension(KEYSTORE_FILE_EXTENSION);

        if !key_file.as_path().exists() {
            return Err(anyhow!("{} key file not found", key_file.to_string_lossy()));
        }

        let file = File::open(&key_file)?;

        let key_entry = serde_json::from_reader(file)?;

        Ok(key_entry)
    }

    pub fn add_key(&mut self, key_name: &str, key_entry: S) -> Result<()> {
        let mut filename = self.path.join(key_name);
        filename.set_extension(KEYSTORE_FILE_EXTENSION);

        let file = File::create(filename)?;

        serde_json::to_writer_pretty(file, &key_entry)?;

        Ok(())
    }

    pub fn _remove_key(&mut self, key_name: &str) -> Result<()> {
        let mut filename = self.path.join(key_name);
        filename.set_extension(KEYSTORE_FILE_EXTENSION);

        fs::remove_file(filename.clone())?;

        Ok(())
    }

    pub fn _keys(&self) -> Result<Vec<(String, S)>> {
        let dir = fs::read_dir(&self.path)?;

        let ext = OsStr::new(KEYSTORE_FILE_EXTENSION);

        dir.into_iter()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension() == Some(ext))
            .flat_map(|path| path.file_stem().map(OsStr::to_owned))
            .flat_map(|stem| stem.to_str().map(ToString::to_string))
            .map(|name| self.get_key(&name).map(|key| (name, key)))
            .collect()
    }
}
