use std::borrow::Cow;

/// Return url which could be used to query this... query
pub trait Query {
    fn query_url(&self) -> Cow<'static, str>;
    fn as_bytes(self) -> Vec<u8>;
}
