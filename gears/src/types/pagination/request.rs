use extensions::pagination::{Pagination, PaginationByKey, PaginationByOffset};
use serde::Deserialize;
use vec1::Vec1;

pub(crate) const QUERY_DEFAULT_LIMIT: u8 = 100;

#[derive(Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum PaginationKind {
    /// key is a value returned in PageResponse.next_key to begin
    /// querying the next page most efficiently. Only one of offset or key
    /// should be set.
    Key { key: Vec1<u8> },
    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key. Only one of offset or key should
    /// be set.
    Offset { offset: u32 },
}

//#[derive(FromForm, Debug)]
#[derive(Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PaginationRequest {
    pub kind: PaginationKind,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    pub limit: u8,
}

impl From<PaginationRequest> for Pagination {
    fn from(PaginationRequest { kind, limit }: PaginationRequest) -> Self {
        match kind {
            PaginationKind::Key { key } => Self::from(PaginationByKey {
                key,
                limit: limit as usize,
            }),
            PaginationKind::Offset { offset } => Self::from(PaginationByOffset {
                offset: offset
                    .checked_mul(limit as u32)
                    .map(|this| this as usize)
                    .unwrap_or(usize::MAX),
                limit: limit as usize,
            }),
        }
    }
}

impl From<core_types::query::request::PageRequest> for PaginationRequest {
    fn from(
        core_types::query::request::PageRequest {
            key,
            offset,
            limit,
            count_total: _,
            reverse: _,
        }: core_types::query::request::PageRequest,
    ) -> Self {
        Self {
            kind: match Vec1::try_from_vec(key) {
                Ok(key) => PaginationKind::Key { key },
                Err(_) => PaginationKind::Offset {
                    offset: offset.try_into().unwrap_or(u32::MAX),
                },
            },
            limit: limit.try_into().unwrap_or(u8::MAX),
        }
    }
}

impl From<PaginationRequest> for core_types::query::request::PageRequest {
    fn from(PaginationRequest { kind, limit }: PaginationRequest) -> Self {
        let (key, offset) = match kind {
            PaginationKind::Key { key } => (key.into_vec(), 0),
            PaginationKind::Offset { offset } => (Vec::new(), offset),
        };
        Self {
            key,
            offset: offset as u64,
            limit: limit as u64,
            count_total: false,
            reverse: false,
        }
    }
}
