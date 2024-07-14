use serde::Deserialize;

use crate::ext::Pagination;

use super::response::PaginationResponse;

pub(crate) const QUERY_DEFAULT_LIMIT: u8 = 100;

//#[derive(FromForm, Debug)]
#[derive(Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PaginationRequest {
    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key. Only one of offset or key should
    /// be set.
    pub offset: u32,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    pub limit: u8,
}

impl From<PaginationRequest> for Pagination {
    fn from(PaginationRequest { offset, limit }: PaginationRequest) -> Self {
        Self {
            offset: offset
                .checked_mul(limit as u32)
                .map(|this| this as usize)
                .unwrap_or(usize::MAX),
            limit: limit as usize,
        }
    }
}

impl PaginationRequest {
    pub fn paginate<T>(
        pagination: Option<Self>,
        iterator: Vec<T>,
    ) -> (Option<PaginationResponse>, Vec<T>) {
        let total = iterator.len();
        let paginate = pagination.is_some();

        let sorted = match pagination {
            Some(PaginationRequest { offset, limit }) => iterator
                .into_iter()
                .skip(
                    offset
                        .checked_mul(limit as u32)
                        .map(|this| this as usize)
                        .unwrap_or(usize::MAX),
                )
                .take(limit as usize)
                .collect(),
            None => iterator,
        };

        (
            match paginate {
                true => Some(PaginationResponse {
                    next_key: Vec::new(), // TODO:ME
                    total: total as u64,
                }),
                false => None,
            },
            sorted,
        )
    }
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: QUERY_DEFAULT_LIMIT,
        }
    }
}

// ParsePagination validate PageRequest and returns page number & limit.
pub fn parse_pagination(PaginationRequest { offset, mut limit }: PaginationRequest) -> (u32, u8) {
    if limit == 0 {
        limit = QUERY_DEFAULT_LIMIT
    }

    let page = offset / (limit as u32) + 1;

    (page, limit)
}

impl From<core_types::query::request::PageRequest> for PaginationRequest {
    fn from(
        core_types::query::request::PageRequest {
            key: _,
            offset,
            limit,
            count_total: _,
            reverse: _,
        }: core_types::query::request::PageRequest,
    ) -> Self {
        Self {
            offset: offset.try_into().unwrap_or(u32::MAX),
            limit: limit.try_into().unwrap_or(u8::MAX),
        }
    }
}

impl From<PaginationRequest> for core_types::query::request::PageRequest {
    fn from(PaginationRequest { offset, limit }: PaginationRequest) -> Self {
        Self {
            key: Vec::new(),
            offset: offset as u64,
            limit: limit as u64,
            count_total: false,
            reverse: false,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_pagination_works() {
        let (page, limit) = parse_pagination(PaginationRequest {
            offset: 100,
            limit: 30,
        });

        assert_eq!(page, 4);
        assert_eq!(limit, 30);

        let (page, limit) = parse_pagination(PaginationRequest {
            offset: 100,
            limit: 0,
        });

        assert_eq!(page, 2);
        assert_eq!(limit, 100);

        let (page, limit) = parse_pagination(PaginationRequest::default());

        assert_eq!(page, 1);
        assert_eq!(limit, 100);
    }
}
