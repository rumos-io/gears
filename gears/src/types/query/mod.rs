pub mod account;
pub mod metadata;

/// Return url which could be used to query this... query
pub trait Query {
    fn query_url(&self) -> &'static str;
    fn into_bytes(self) -> Vec<u8>;
}
