use serde::Deserialize;

const QUERY_DEFAULT_LIMIT: u8 = 100;

//#[derive(FromForm, Debug)]
#[derive(Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Pagination {
    offset: Option<u32>,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    limit: Option<u8>,
}

// ParsePagination validate PageRequest and returns page number & limit.
pub fn parse_pagination(pagination: Pagination) -> (u32, u8) {
    let offset = pagination.offset.unwrap_or(0);
    let mut limit = pagination.limit.unwrap_or(QUERY_DEFAULT_LIMIT);

    if limit == 0 {
        limit = QUERY_DEFAULT_LIMIT
    }

    let page = offset / (limit as u32) + 1;

    (page, limit)
}

impl From<core_types::query::request::PageRequest> for Pagination {
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
            offset: Some(offset as u32),
            limit: Some(limit as u8),
        }
    }
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
