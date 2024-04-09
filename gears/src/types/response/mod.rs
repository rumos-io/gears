pub mod any;
use serde::{Deserialize, Serialize};

pub mod tx;
pub mod tx_event;

mod inner {
    pub use ibc_types::query::response::PageResponse;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageResponse {
    pub next_key: Vec<u8>,
    pub total: u64,
}

impl From<inner::PageResponse> for PageResponse {
    fn from(inner::PageResponse { next_key, total }: inner::PageResponse) -> Self {
        Self { next_key, total }
    }
}

impl From<PageResponse> for inner::PageResponse {
    fn from(PageResponse { next_key, total }: PageResponse) -> Self {
        Self { next_key, total }
    }
}
