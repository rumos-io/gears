use std::{borrow::Cow, ops::Bound};

use chrono::{DateTime, Utc};
use gears::{
    store::database::Database,
    tendermint::types::time::Timestamp,
    types::store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
};

use super::{parse_proposal_key_bytes, Proposal};

#[derive(Debug)]
pub struct InactiveProposalIterator<'a, DB>(StoreRange<'a, DB>);

impl<'a, DB: Database> InactiveProposalIterator<'a, DB> {
    pub fn new(store: Store<'a, DB>, end_time: &Timestamp) -> InactiveProposalIterator<'a, DB> {
        Self(
            store.into_range((
                Bound::Included(Proposal::KEY_INACTIVE_QUEUE_PREFIX.to_vec()),
                Bound::Excluded(
                    [
                        Proposal::KEY_INACTIVE_QUEUE_PREFIX.as_slice(),
                        end_time.format_bytes_rounded().as_slice(),
                    ]
                    .concat()
                    .to_vec(),
                ),
            )),
        )
    }
}

impl<'a, DB: Database> Iterator for InactiveProposalIterator<'a, DB> {
    type Item = Result<((u64, DateTime<Utc>), Cow<'a, Vec<u8>>), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(var) = self.0.next() {
            match var {
                Ok((key, value)) => Some(Ok((parse_proposal_key_bytes(key.as_ref()), value))),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}
