use std::{collections::HashMap, fmt::Debug};

use gears::context::InfallibleContextMut;

use crate::types::plan::Plan;

pub trait UpgradeHandler: Debug + Clone + Send + Sync + 'static {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB, SK, M>(
        &self,
        ctx: &mut CTX,
        plan: &Plan,
        versions: impl IntoIterator<Item = (M, u64)>,
    ) -> anyhow::Result<HashMap<M, u64>>;
}

#[derive(Debug, Clone)]
pub struct DummyHandler;

impl UpgradeHandler for DummyHandler {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB, SK, M>(
        &self,
        _ctx: &mut CTX,
        _plan: &Plan,
        _versions: impl IntoIterator<Item = (M, u64)>,
    ) -> anyhow::Result<HashMap<M, u64>> {
        Ok(HashMap::new())
    }
}
