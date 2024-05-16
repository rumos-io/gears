#![warn(rust_2018_idioms)]

pub mod error;
pub mod ext;
mod memory;
pub mod prefix;
#[cfg(feature = "rocksdb")]
mod rocks;

pub use memory::*;
#[cfg(feature = "rocksdb")]
pub use rocks::*;

pub trait Database {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    fn put(&self, key: Vec<u8>, value: Vec<u8>);

    fn iterator<'a>(&'a self) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;

    fn prefix_iterator<'a>(
        &'a self,
        prefix: Vec<u8>,
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;
}
