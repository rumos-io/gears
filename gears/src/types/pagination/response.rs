use extensions::pagination::{PaginationKey, PaginationResultElement};
use serde::{Deserialize, Serialize};

mod inner {
    pub use core_types::query::response::PageResponse;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginationResponse {
    pub next_key: Vec<u8>,
    pub total: u64,
}

impl PaginationResponse {
    pub fn new(total: usize, key: impl Into<Vec<u8>>) -> Self {
        Self {
            next_key: key.into(),
            total: total as u64,
        }
    }
}

impl From<inner::PageResponse> for PaginationResponse {
    fn from(inner::PageResponse { next_key, total }: inner::PageResponse) -> Self {
        Self { next_key, total }
    }
}

impl From<PaginationResponse> for inner::PageResponse {
    fn from(PaginationResponse { next_key, total }: PaginationResponse) -> Self {
        Self { next_key, total }
    }
}

impl<T: PaginationKey> From<PaginationResultElement<T>> for PaginationResponse {
    fn from(
        PaginationResultElement {
            total,
            next_key: next_element,
        }: PaginationResultElement<T>,
    ) -> Self {
        Self {
            next_key: next_element
                .map(|this| this.iterator_key().into_owned())
                .unwrap_or_default(),
            total: total as u64,
        }
    }
}
