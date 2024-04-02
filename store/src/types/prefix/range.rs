use std::ops::Bound;

use database::Database;
use trees::iavl::Range;

pub struct PrefixRange<'a, DB: Database> {
    pub(super) parent_range: Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB>,
    pub(super) prefix_length: usize,
}

impl<'a, DB: Database> Iterator for PrefixRange<'a, DB> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.parent_range.next()?;

        // Remove the prefix from the key - this is safe since all returned keys will include the prefix.
        // TODO: what if the key now has zero length, is this safe given the check on KVStore set.
        let truncated_key = next.0[self.prefix_length..].to_vec();

        Some((truncated_key, next.1))
    }
}
