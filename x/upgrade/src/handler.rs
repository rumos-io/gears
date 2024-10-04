use std::fmt::Debug;

use gears::context::InfallibleContextMut;

use crate::types::plan::Plan;

pub trait UpgradeHandler: Debug + Clone + Send + Sync + 'static {
    fn handle<CTX: InfallibleContextMut<DB, SK>, DB, SK>(
        &self,
        ctx: &mut CTX,
        plan: &Plan,
        versions: impl IntoIterator<Item = ()>,
    );
}
