mod error;
mod memory;
mod prefix;
mod rocks;

pub use memory::*;
pub use prefix::*;
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
