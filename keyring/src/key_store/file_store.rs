use std::{
    fs::{self, remove_file, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use std::fs::File;

use crate::{error::Error, key_pair::KeyPair};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use std::os::unix::fs::PermissionsExt;

/// A key store that stores keys in files.
#[derive(Debug)]
pub struct FileStore {
    path: PathBuf,
    password: Option<String>, //TODO: needs to be a secret
}

pub const PEM_EXTENSION: &str = "pem";
const KEY_HASH_FILE: &str = "key_hash";

impl FileStore {
    /// Opens an existing keyring or creates a new one if `create` is `true`.
    pub fn open(path: PathBuf, password: Option<String>, create: bool) -> Result<Self, Error> {
        let key_hash_path = path.join(KEY_HASH_FILE);

        match fs::read_to_string(&key_hash_path) {
            Ok(password_hash) => {
                Self::verify_password(password.as_deref(), &password_hash, &key_hash_path)?;
                Ok(Self { path, password })
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    if create {
                        fs::create_dir_all(&path).map_err(|e| Error::IO {
                            msg: e.to_string(),
                            source: e,
                            path: path.display().to_string(),
                        })?;

                        let password_hash = Self::calculate_password_hash(password.as_deref())?;

                        let mut file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(&key_hash_path)
                            .map_err(|e| Error::IO {
                                msg: e.to_string(),
                                source: e,
                                path: path.display().to_string(),
                            })?;

                        file.write_all(&password_hash.as_bytes())
                            .map_err(|e| Error::IO {
                                msg: e.to_string(),
                                source: e,
                                path: path.display().to_string(),
                            })?;

                        Self::set_readonly(file, key_hash_path.display().to_string())?;

                        Ok(Self { path, password })
                    } else {
                        Err(Error::KeyringDoesNotExist(path.display().to_string()))
                    }
                } else {
                    Err(Error::IO {
                        msg: e.to_string(),
                        source: e,
                        path: key_hash_path.display().to_string(),
                    })
                }
            }
        }
    }

    fn verify_password(
        password: Option<&str>,
        password_hash: &str,
        key_hash_path: &Path,
    ) -> Result<(), Error> {
        match password {
            Some(password) => {
                let parsed_hash =
                    PasswordHash::new(&password_hash).map_err(|e| Error::KeyHash {
                        source: e,
                        path: key_hash_path.display().to_string(),
                        msg: e.to_string(),
                    })?;

                // NOTE: hash params from `parsed_hash` are used instead of what is configured in the
                // `Argon2` instance.
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed_hash)
                    .map_err(|_| Error::IncorrectPassword)
            }
            None => {
                if password_hash.is_empty() {
                    Ok(())
                } else {
                    Err(Error::IncorrectPassword)
                }
            }
        }
    }

    fn calculate_password_hash(password: Option<&str>) -> Result<String, Error> {
        if let Some(password) = password {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            Ok(argon2
                .hash_password(password.as_ref(), &salt)
                .map_err(|e| Error::InvalidPassword {
                    source: e,
                    msg: e.to_string(),
                })?
                .to_string())
        } else {
            return Ok("".into());
        }
    }

    /// Returns an [`Error`] if an entry with the same name already exists. If an entry already exists for
    /// the given key but with a different name then a new separate entry will be created.
    pub fn set_key_pair<S: AsRef<str>>(&self, key_name: S, key_pair: KeyPair) -> Result<(), Error> {
        let mut path = self.path.join(key_name.as_ref());
        path.set_extension(PEM_EXTENSION);

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| {
                if e.kind() == ErrorKind::AlreadyExists {
                    Error::AlreadyExists {
                        name: key_name.as_ref().into(),
                        location: path.display().to_string(),
                    }
                } else {
                    Error::IO {
                        msg: e.to_string(),
                        source: e,
                        path: path.display().to_string(),
                    }
                }
            })?;

        let key = match &self.password {
            Some(password) => key_pair.to_pkcs8_encrypted_pem(password),
            None => key_pair.to_pkcs8_pem(),
        };

        file.write_all(&key.as_bytes()).map_err(|e| Error::IO {
            msg: e.to_string(),
            source: e,
            path: path.display().to_string(),
        })?;

        Self::set_readonly(file, path.display().to_string())
    }

    fn set_readonly(file: File, path: String) -> Result<(), Error> {
        let mut perms = file
            .metadata()
            .map_err(|e| Error::ReadOnly {
                msg: e.to_string(),
                source: e,
                path: path.clone(),
            })?
            .permissions();

        #[cfg(unix)]
        {
            PermissionsExt::set_mode(&mut perms, 0o400);
        }

        #[cfg(not(unix))]
        {
            perms.set_readonly(true);
        }

        file.set_permissions(perms).map_err(|e| Error::ReadOnly {
            msg: e.to_string(),
            source: e,
            path,
        })?;
        Ok(())
    }

    /// Gets the entry with the given name.
    /// Returns [`Error`] if no entry with the given name can be found.
    pub fn get_key_by_name<S>(&self, name: S) -> Result<KeyPair, Error>
    where
        S: AsRef<str>,
    {
        let mut path = self.path.join(name.as_ref());
        path.set_extension(PEM_EXTENSION);

        fs::read(&path)
            .map_err(|e| {
                if e.kind() == ErrorKind::NotFound {
                    Error::DoesNotExist {
                        name: name.as_ref().into(),
                        location: path.display().to_string(),
                    }
                } else {
                    Error::IO {
                        msg: e.to_string(),
                        source: e,
                        path: path.display().to_string(),
                    }
                }
            })
            .and_then(|v| {
                let raw_key = String::from_utf8(v).map_err(|e| Error::InvalidUTF8 {
                    msg: e.to_string(),
                    source: e,
                    path: path.display().to_string(),
                })?;

                if let Some(password) = &self.password {
                    KeyPair::from_pkcs8_encrypted_pem(&raw_key, password)
                } else {
                    KeyPair::from_pkcs8_pem(&raw_key)
                }
                .map_err(|e| Error::PKCS8 {
                    source: e,
                    path: path.display().to_string(),
                    msg: e.to_string(),
                })
            })
    }

    /// Deletes the entry with the given name.
    /// Returns [`Error`] if no entry with the given name can be found.
    pub fn delete_key_by_name<S>(&self, name: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        let mut path = self.path.join(name.as_ref());
        path.set_extension(PEM_EXTENSION);

        remove_file(&path).map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                Error::DoesNotExist {
                    name: name.as_ref().into(),
                    location: path.display().to_string(),
                }
            } else {
                Error::IO {
                    msg: e.to_string(),
                    source: e,
                    path: path.display().to_string(),
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {

    use std::path;

    use super::*;

    #[test]
    fn calc_none_password_hash() {
        let password_hash = FileStore::calculate_password_hash(None).expect("password is valid");
        assert_eq!(&password_hash, "");
    }

    #[test]
    fn calc_password_hash() {
        let password = "password";

        let password_hash =
            FileStore::calculate_password_hash(Some(password)).expect("password is valid");
        assert_eq!(&password_hash[0..31], "$argon2id$v=19$m=19456,t=2,p=1$");
    }

    #[test]
    fn verify_none_password_success() {
        let path = path::PathBuf::from("test");
        FileStore::verify_password(None, "", &path).expect("password is valid");
    }

    #[test]
    fn verify_password_success() {
        let password = "pastword";
        let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$0Wzy7RqEZA/HDKXNHAy6lQ$QAhM6YYWd5ZLZcgcMXYIRztBL0IfyHjacsF6X4kbYR0";
        let path = path::PathBuf::from("test");

        FileStore::verify_password(Some(password), password_hash, &path)
            .expect("password is valid");
    }

    #[test]
    fn verify_password_wrong_password() {
        let password = "wrong_password";
        let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$0Wzy7RqEZA/HDKXNHAy6lQ$QAhM6YYWd5ZLZcgcMXYIRztBL0IfyHjacsF6X4kbYR0";
        let path = path::PathBuf::from("test");

        let err = FileStore::verify_password(Some(password), &password_hash, &path).unwrap_err();

        assert!(matches!(err, Error::IncorrectPassword));
    }

    #[test]
    fn verify_password_invalid_hash() {
        let password = "pastword";
        let password_hash = "1vcy/0FkRHslP417Yk0VtQ$aIBL2rzqEOiYUPQbHXuPr6Nz9a0zPImBtmNPu1FSVfw";
        let path = path::PathBuf::from("test");

        let err = FileStore::verify_password(Some(password), password_hash, &path).unwrap_err();
        assert!(matches!(err, Error::KeyHash { .. }));
    }

    #[test]
    fn e2e_password_hash() {
        let password = "password123";
        let path = path::PathBuf::from("test");

        let password_hash =
            FileStore::calculate_password_hash(Some(password)).expect("password is valid");
        FileStore::verify_password(Some(password), &password_hash, &path)
            .expect("password matches hash");
    }
}
