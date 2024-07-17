use std::str::FromStr;

use clap::ArgAction;
use core_types::errors::CoreError;
use vec1::Vec1;

use crate::types::pagination::request::{PaginationKind, PaginationRequest, QUERY_DEFAULT_LIMIT};

#[derive(Debug, Clone)]
pub struct CliVec1(Vec1<u8>);

impl FromStr for CliVec1 {
    type Err = vec1::Size0Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Vec1::try_from_vec(Vec::<u8>::from(s))?))
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct CliPaginationRequest {
    /// key is a value returned in PageResponse.next_key to begin
    /// querying the next page most efficiently. Only one of offset or key
    /// should be set.
    #[arg(short, long, action = ArgAction::Set , help_heading = "Pagination")]
    pub key: Option<CliVec1>,
    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key. Only one of offset or key should
    /// be set.
    #[arg(short, long, default_value = "0", required = false, action = ArgAction::Set , help_heading = "Pagination")]
    pub offset: Option<u32>,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    #[arg(short, long, default_value_t = QUERY_DEFAULT_LIMIT, action = ArgAction::Set, help_heading = "Pagination")]
    pub limit: u8,
}

impl From<PaginationRequest> for CliPaginationRequest {
    fn from(PaginationRequest { kind, limit }: PaginationRequest) -> Self {
        match kind {
            PaginationKind::Key { key } => Self {
                limit,
                key: Some(CliVec1(key)),
                offset: None,
            },
            PaginationKind::Offset { offset } => Self {
                limit,
                key: None,
                offset: Some(offset),
            },
        }
    }
}

impl TryFrom<CliPaginationRequest> for PaginationRequest {
    type Error = CoreError;

    fn try_from(
        CliPaginationRequest { key, offset, limit }: CliPaginationRequest,
    ) -> Result<Self, Self::Error> {
        match (key, offset) {
            (None, None) => Ok(Self {
                kind: PaginationKind::Offset { offset: 0 },
                limit,
            }),
            (None, Some(offset)) => Ok(Self {
                kind: PaginationKind::Offset { offset },
                limit,
            }),
            (Some(key), None) => Ok(Self {
                kind: PaginationKind::Key { key: key.0 },
                limit,
            }),
            (Some(_), Some(_)) => Err(CoreError::DecodeGeneral(
                "`offset` and `key` exclusive to each other".to_owned(),
            )),
        }
    }
}
