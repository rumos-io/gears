use std::{collections::HashMap, fmt::Debug};

use gears::context::InfallibleContextMut;

use crate::types::plan::Plan;

pub trait UpgradeHandler: Debug + Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn handle<CTX: InfallibleContextMut<DB, SK>, DB, SK, M>(
        &self,
        ctx: &mut CTX,
        plan: &Plan,
        versions: impl IntoIterator<Item = (M, u64)>,
    ) -> anyhow::Result<HashMap<M, u64>>;
}

pub mod dummy {

    use super::*;

    #[derive(Debug, Clone, strum::EnumIter)]
    pub enum NullUpgradeHandler {}

    impl UpgradeHandler for NullUpgradeHandler {
        fn name(&self) -> &'static str {
            unreachable!()
        }

        fn handle<CTX: InfallibleContextMut<DB, SK>, DB, SK, M>(
            &self,
            _ctx: &mut CTX,
            _plan: &Plan,
            _versions: impl IntoIterator<Item = (M, u64)>,
        ) -> anyhow::Result<HashMap<M, u64>> {
            unreachable!()
        }
    }
}
