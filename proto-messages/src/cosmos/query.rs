use std::borrow::Cow;

/// Return url which could be used to query this... query
pub trait QueryUrl {
    fn query_url() -> Cow<'static, str>;
}
