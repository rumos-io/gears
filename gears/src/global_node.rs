use std::sync::OnceLock;

use database::Database;
use tendermint::{
    application::ABCIApplication,
    types::{request::deliver_tx::RequestDeliverTx, response::deliver_tx::ResponseDeliverTx},
};

use crate::{
    application::{handlers::node::ABCIHandler, ApplicationInfo},
    params::ParamsSubspaceKey,
};

// I better use unsafe that this. This is awful
pub(crate) static GLOBAL_NODE: OnceLock<Box<dyn GlobalNode>> = OnceLock::new();

pub fn global_node() -> Option<&'static dyn GlobalNode> {
    GLOBAL_NODE.get().map(|this| &**this)
}

pub trait GlobalNode: Sync + Send + 'static {
    fn deliver_tx(&self, req: RequestDeliverTx) -> ResponseDeliverTx;
}

impl<DB: Database, PSK: ParamsSubspaceKey, H: ABCIHandler, AI: ApplicationInfo> GlobalNode
    for crate::baseapp::BaseApp<DB, PSK, H, AI>
{
    fn deliver_tx(&self, req: RequestDeliverTx) -> ResponseDeliverTx {
        <Self as ABCIApplication<H::Genesis>>::deliver_tx(self, req)
    }
}
