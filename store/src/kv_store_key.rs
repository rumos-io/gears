use std::borrow::Borrow;

/// Simple struct to use if you need to pass const value of bytes without any additional info for key
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SimpleKvStoreKey(pub KeyBytes);

impl TryFrom<Vec<u8>> for SimpleKvStoreKey {
    type Error = KeyError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl TryFrom<&[u8]> for SimpleKvStoreKey {
    type Error = KeyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl KvStoreKey for SimpleKvStoreKey {
    fn prefix(self) -> KeyBytes {
        self.0
    }
}

pub trait KvStoreKey {
    fn prefix(self) -> KeyBytes;
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("Key should be not empty")]
pub struct KeyError;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct KeyBytes(Vec<u8>);

impl KeyBytes {
    pub fn new(bytes: impl IntoIterator<Item = u8>) -> Result<Self, KeyError> {
        let bytes = bytes.into_iter().collect::<Vec<u8>>();

        if bytes.is_empty() {
            Err(KeyError)
        } else {
            Ok(Self(bytes))
        }
    }

    pub fn from_ref(slice: &(impl AsRef<[u8]> + ?Sized)) -> Result<Self, KeyError> {
        Self::new(slice.as_ref().into_iter().cloned())
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl IntoIterator for KeyBytes {
    type Item = u8;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl AsRef<[u8]> for KeyBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<Vec<u8>> for KeyBytes {
    fn as_ref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl Borrow<Vec<u8>> for KeyBytes {
    fn borrow(&self) -> &Vec<u8> {
        &self.0
    }
}

impl Borrow<[u8]> for KeyBytes {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<Vec<u8>> for KeyBytes {
    type Error = KeyError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&[u8]> for KeyBytes {
    type Error = KeyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::new(value.into_iter().cloned())
    }
}
