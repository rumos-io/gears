use database::Database;
use kv_store::StoreKey;

use crate::{
    context::{InfallibleContext, InfallibleContextMut, QueryableContext, TransactionalContext},
    params::{ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
    types::store::gas::errors::GasStoreErrors,
};

pub trait ParamsKeeper<PSK: ParamsSubspaceKey> {
    type Param: ParamsSerialize + ParamsDeserialize;

    fn get<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Self::Param;

    fn try_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Self::Param, GasStoreErrors>;

    fn set<DB: Database, SK: StoreKey, KV: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: Self::Param,
    );

    fn try_set<DB: Database, SK: StoreKey, KV: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: Self::Param,
    ) -> Result<(), GasStoreErrors>;
}
