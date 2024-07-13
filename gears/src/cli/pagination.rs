use clap::ArgAction;

use crate::rest::request::{PaginationRequest, QUERY_DEFAULT_LIMIT};

#[derive(Debug, Clone, clap::Args)]
pub struct CliPaginationRequest {
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
        }
    }
}

impl From<PaginationRequest> for CliPaginationRequest {
    fn from(PaginationRequest { offset, limit }: PaginationRequest) -> Self {
        Self { offset, limit }
    }
}

impl From<CliPaginationRequest> for PaginationRequest {
    fn from(CliPaginationRequest { offset, limit }: CliPaginationRequest) -> Self {
        Self { offset, limit }
    }
}
