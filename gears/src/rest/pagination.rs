use crate::types::pagination::request::{PaginationKind, PaginationRequest, QUERY_DEFAULT_LIMIT};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Pagination {
    /*
    ! NOTE: this lines of code https://github.com/NYBACHOK/gears/blob/81883ecfd28c65c8b90460fd9515b18ed33094a4/gears/src/rest/handlers.rs#L69-L76
    ! doesn't have any way to use key instead of offset, so I assume it not possible at all for this moment
    */
    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key. Only one of offset or key should
    /// be set.
    offset: Option<u32>,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    limit: Option<u8>,
}

impl From<Pagination> for PaginationRequest {
    fn from(pagination: Pagination) -> Self {
        let (offset, limit) = parse_pagination(pagination);

        Self {
            limit,
            kind: PaginationKind::Offset { offset },
        }
    }
}

// ParsePagination validate Pagination and returns page number & limit.
pub fn parse_pagination(pagination: Pagination) -> (u32, u8) {
    let offset = pagination.offset.unwrap_or(0);
    let mut limit = pagination.limit.unwrap_or(QUERY_DEFAULT_LIMIT);

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
            offset: Some(100),
            limit: Some(30),
        });

        assert_eq!(page, 4);
        assert_eq!(limit, 30);

        let (page, limit) = parse_pagination(Pagination {
            offset: Some(100),
            limit: Some(0),
        });

        assert_eq!(page, 2);
        assert_eq!(limit, 100);

        let (page, limit) = parse_pagination(Pagination {
            offset: None,
            limit: None,
        });

        assert_eq!(page, 1);
        assert_eq!(limit, 100);
    }
}
