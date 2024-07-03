/// paginate returns the correct starting and ending index for a paginated query,
/// given that client provides a desired page and limit of objects and the handler
/// provides the total number of objects. The start page is assumed to be 1-indexed.
/// If the start page is invalid, non-positive values are returned signaling the
/// request is invalid; it returns non-positive values if limit is non-positive and
/// defLimit is negative.
// TODO: move to some place where it can be reused for queries
pub fn paginate(
    num_objs: u64,
    page: u64,
    mut limit: u64,
    default_limit: u64,
) -> Option<(u64, u64)> {
    if page == 0 {
        // invalid start page
        return None;
    }

    // fallback to default limit if supplied limit is invalid
    if limit == 0 {
        if default_limit == 0 {
            // invalid default limit
            return None;
        }
        limit = default_limit;
    }

    let start = (page - 1) * limit;
    let mut end = limit + start;

    if end >= num_objs {
        end = num_objs;
    }

    if start >= num_objs {
        // page is out of bounds
        return None;
    }

    Some((start, end))
}
