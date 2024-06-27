use std::marker::PhantomData;

use database::Database;
use kv_store::StoreKey;

use crate::{
    application::keepers::params::ParamsKeeper,
    context::TransactionalContext,
    params::{gas::subspace_mut, ParamsSubspaceKey},
    x::submission::param::ParamChange,
};

use super::{SubmissionCheckHandler, SubmissionHandler};

#[derive(Debug)]
pub struct ParamChangeSubmissionHandler<PK>(PhantomData<PK>);

impl<PSK: ParamsSubspaceKey, PK: ParamsKeeper<PSK>> SubmissionHandler<PSK, ParamChange<PSK>>
    for ParamChangeSubmissionHandler<PK>
{
    fn handle<CTX: TransactionalContext<DB, SK>, DB: Database, SK: StoreKey>(
        proposal: ParamChange<PSK>,
        ctx: &mut CTX,
        subspace_key: &PSK,
    ) -> anyhow::Result<()> {
        if !Self::submission_check::<PK>(&proposal) {
            Err(anyhow::anyhow!(
                "Can't handle this proposal: no such keys in subspace"
            ))?
        }

        let mut store = subspace_mut(ctx, subspace_key);

        store.raw_key_set(proposal.key, proposal.value)?;

        Ok(())
    }
}
