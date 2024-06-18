use std::{borrow::Cow, ops::Bound};

use chrono::{DateTime, SubsecRound, Utc};
use gears::{
    store::database::Database,
    types::store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
};

use super::{parse_proposal_key_bytes, Proposal, SORTABLE_DATE_TIME_FORMAT};

#[derive(Debug)]
pub struct InactiveProposalIterator<'a, DB>(StoreRange<'a, DB>);

impl<'a, DB: Database> InactiveProposalIterator<'a, DB> {
    pub fn new(
        store: &'a Store<'a, DB>,
        end_time: &DateTime<Utc>,
    ) -> InactiveProposalIterator<'a, DB> {
        Self(
            store.range((
                Bound::Included(Proposal::KEY_INACTIVE_QUEUE_PREFIX.to_vec()),
                Bound::Excluded(
                    [
                        Proposal::KEY_INACTIVE_QUEUE_PREFIX.as_slice(),
                        end_time
                            .round_subsecs(0)
                            .format(SORTABLE_DATE_TIME_FORMAT)
                            .to_string()
                            .as_bytes(),
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
