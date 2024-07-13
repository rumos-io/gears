use serde::Deserialize;

const QUERY_DEFAULT_LIMIT: u8 = 100;

//#[derive(FromForm, Debug)]
#[derive(Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PaginationRequest {
    offset: u32,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    limit: u8,
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
