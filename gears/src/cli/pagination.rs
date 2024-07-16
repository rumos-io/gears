use clap::ArgAction;

use crate::{
    ext::{Pagination, PaginationByKey, PaginationByOffset},
    types::pagination::request::{PaginationRequest, QUERY_DEFAULT_LIMIT},
};

#[derive(Debug, Clone, clap::Args)]
pub struct CliPaginationRequest {
    /// key is a value returned in PageResponse.next_key to begin
    /// querying the next page most efficiently. Only one of offset or key
    /// should be set.
    #[arg(short, long, action = ArgAction::Set , help_heading = "Pagination")]
    pub key: Option<Vec<u8>>,
    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key. Only one of offset or key should
    /// be set.
    #[arg(short, long, default_value_t = 0, action = ArgAction::Set , help_heading = "Pagination")]
    pub offset: u32,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    #[arg(short, long, default_value_t = QUERY_DEFAULT_LIMIT, action = ArgAction::Set, help_heading = "Pagination")]
    pub limit: u8,
}

impl Default for CliPaginationRequest {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: QUERY_DEFAULT_LIMIT,
            key: None,
        }
    }
}

impl From<PaginationRequest> for CliPaginationRequest {
    fn from(PaginationRequest { offset, limit, key }: PaginationRequest) -> Self {
        Self { offset, limit, key }
    }
}

impl From<CliPaginationRequest> for PaginationRequest {
    fn from(CliPaginationRequest { offset, limit, key }: CliPaginationRequest) -> Self {
        Self { offset, limit, key }
    }
}

impl From<CliPaginationRequest> for Pagination {
    fn from(CliPaginationRequest { offset, limit, key }: CliPaginationRequest) -> Self {
        match key {
            Some(key) => Self::from(PaginationByKey {
                key,
                limit: limit as usize,
            }),
            None => Self::from(PaginationByOffset {
                offset: offset
                    .checked_mul(limit as u32)
                    .map(|this| this as usize)
                    .unwrap_or(usize::MAX),
                limit: limit as usize,
            }),
        }
    }
}
