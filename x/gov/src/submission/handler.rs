use std::marker::PhantomData;

use gears::context::TransactionalContext;
use gears::params::gas::subspace_mut;
use gears::store::database::Database;
use gears::store::StoreKey;

use gears::{
    application::keepers::params::ParamsKeeper, context::InfallibleContextMut,
    params::ParamsSubspaceKey,
};

use super::param::ParamChange;

pub trait SubmissionHandler<PK: ParamsKeeper<PSK>, PSK: ParamsSubspaceKey, P> {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB: Database, SK: StoreKey>(
        proposal: P,
        ctx: &mut CTX,
        subspace: &PSK,
    ) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct ParamChangeSubmissionHandler<PK>(PhantomData<PK>);

impl<PSK: ParamsSubspaceKey, PK: ParamsKeeper<PSK>> SubmissionHandler<PK, PSK, ParamChange<PSK>>
    for ParamChangeSubmissionHandler<PK>
{
    fn handle<CTX: TransactionalContext<DB, SK>, DB: Database, SK: StoreKey>(
        proposal: ParamChange<PSK>,
        ctx: &mut CTX,
        subspace_key: &PSK,
    ) -> anyhow::Result<()> {
        if !PK::check_key(&proposal.key) {
            Err(anyhow::anyhow!(
                "Can't handle this proposal: no such keys in subspace"
            ))?
        }

        if !PK::validate(&proposal.key, &proposal.value) {
            Err(anyhow::anyhow!("Can't handle this proposal: invalid bytes"))?
        }

        let mut store = subspace_mut(ctx, subspace_key);

        // This block is still safe, but I use it to make sure to indicate that this method could break any 
        unsafe {
            store.raw_key_set(proposal.key, proposal.value)?;
        }

        Ok(())
    }
}
