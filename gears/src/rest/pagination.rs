use crate::types::pagination::request::{PaginationRequest, QUERY_DEFAULT_LIMIT};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Pagination {
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
        Self { offset, limit }
    }
}

impl From<Pagination> for PaginationRequest {
    fn from(Pagination { offset, limit }: Pagination) -> Self {
        Self { offset, limit }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: QUERY_DEFAULT_LIMIT,
        }
    }
}

// ParsePagination validate PageRequest and returns page number & limit.
pub fn parse_pagination(Pagination { offset, mut limit }: Pagination) -> (u32, u8) {
    if limit == 0 {
        limit = QUERY_DEFAULT_LIMIT
    }

    let page = offset / (limit as u32) + 1;

    (page, limit)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_pagination_works() {
        let (page, limit) = parse_pagination(Pagination {
            offset: 100,
            limit: 30,
        });

        assert_eq!(page, 4);
        assert_eq!(limit, 30);

        let (page, limit) = parse_pagination(Pagination {
            offset: 100,
            limit: 0,
        });

        assert_eq!(page, 2);
        assert_eq!(limit, 100);

        let (page, limit) = parse_pagination(Pagination::default());

        assert_eq!(page, 1);
        assert_eq!(limit, 100);
    }
}
