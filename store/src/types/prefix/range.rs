use std::borrow::Cow;

use crate::utils::MergedRange;

pub struct PrefixRange<'a> {
    pub(super) parent_range: MergedRange<'a>,
    pub(super) prefix_length: usize,
}

impl<'a> Iterator for PrefixRange<'a> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        let (first, second) = self.parent_range.next()?;

        // Remove the prefix from the key - this is safe since all returned keys will include the prefix.
        // TODO: what if the key now has zero length, is this safe given the check on KVStore set.
        let truncated_key = first[self.prefix_length..].to_vec();

        Some((Cow::Owned(truncated_key), second))
    }
}
