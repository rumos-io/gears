use crate::Evidence;
use anyhow::anyhow;
use gears::{
    context::{block::BlockContext, init::InitContext, tx::TxContext},
    store::{database::Database, StoreKey},
};
use std::{collections::HashMap, rc::Rc};

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub enum HandlerFn<CTX, E> {
    NonMut(Rc<dyn Fn(&CTX, &E) -> anyhow::Result<()>>),
    Mut(Rc<dyn Fn(&mut CTX, &E) -> anyhow::Result<()>>),
}

impl<CTX, E> HandlerFn<CTX, E> {
    pub fn from_mut<F: Fn(&mut CTX, &E) -> anyhow::Result<()> + 'static>(f: F) -> Self {
        Self::Mut(Rc::new(f))
    }
    pub fn from_non_mut<F: Fn(&CTX, &E) -> anyhow::Result<()> + 'static>(f: F) -> Self {
        Self::NonMut(Rc::new(f))
    }

    pub fn call_mut(&self, ctx: &mut CTX, evidence: &E) -> anyhow::Result<()> {
        if let HandlerFn::Mut(func) = self {
            func(ctx, evidence)
        } else {
            Err(anyhow!("mutable handler is not implemented"))
        }
    }

    pub fn call_non_mut(&self, ctx: &mut CTX, evidence: &E) -> anyhow::Result<()> {
        if let HandlerFn::NonMut(func) = self {
            func(ctx, evidence)
        } else {
            Err(anyhow!("non mutable handler is not implemented"))
        }
    }
}

/// Handler defines an agnostic Evidence handler. The handler is responsible
/// for executing all corresponding business logic necessary for verifying the
/// evidence as valid. In addition, the Handler may execute any necessary
/// slashing and potential jailing.

// this ugly api is used because it allows to skip multiple ctx generics in declaration of keeper
// and abci handler
// we have to provide generics in the abci constructor and I think that it may be annoying
//
// TODO: think how to improve the router
pub enum Handler<'a, DB, SK, E>
where
    E: Evidence,
    SK: StoreKey,
    DB: Database,
{
    Init(HandlerFn<InitContext<'a, DB, SK>, E>),
    Tx(HandlerFn<TxContext<'a, DB, SK>, E>),
    Block(HandlerFn<BlockContext<'a, DB, SK>, E>),
}

impl<'a, DB, SK, E> Handler<'a, DB, SK, E>
where
    E: Evidence,
    SK: StoreKey,
    DB: Database,
{
    pub fn new_init(handler_fn: HandlerFn<InitContext<'a, DB, SK>, E>) -> Self {
        Self::Init(handler_fn)
    }
    pub fn new_tx(handler_fn: HandlerFn<TxContext<'a, DB, SK>, E>) -> Self {
        Self::Tx(handler_fn)
    }
    pub fn new_block(handler_fn: HandlerFn<BlockContext<'a, DB, SK>, E>) -> Self {
        Self::Block(handler_fn)
    }

    pub fn handle_with_init_ctx(
        &self,
        ctx: &mut InitContext<'a, DB, SK>,
        evidence: &E,
    ) -> anyhow::Result<()> {
        match self {
            Handler::Init(func) => func.call_mut(ctx, evidence),
            _ => Err(anyhow!("incompatible context type")),
        }
    }

    pub fn handle_with_tx_ctx(
        &self,
        ctx: &mut TxContext<'a, DB, SK>,
        evidence: &E,
    ) -> anyhow::Result<()> {
        match self {
            Handler::Tx(func) => func.call_mut(ctx, evidence),
            _ => Err(anyhow!("incompatible context type")),
        }
    }

    pub fn handle_with_block_ctx(
        &self,
        ctx: &mut BlockContext<'a, DB, SK>,
        evidence: &E,
    ) -> anyhow::Result<()> {
        match self {
            Handler::Block(func) => func.call_mut(ctx, evidence),
            _ => Err(anyhow!("incompatible context type")),
        }
    }
}

impl<'a, DB, SK, E> std::fmt::Debug for Handler<'a, DB, SK, E>
where
    E: Evidence,
    SK: StoreKey,
    DB: Database,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fn(&mut impl TransactionalContext, &impl Evidence)")
    }
}

#[derive(Debug)]
pub struct Router<'a, DB, SK, E>
where
    E: Evidence,
    SK: StoreKey,
    DB: Database,
{
    routes: HashMap<String, Handler<'a, DB, SK, E>>,
}

/// Router defines a contract for which any Evidence handling module must
/// implement in order to route Evidence to registered Handlers.
impl<'a, DB, SK, E> Router<'a, DB, SK, E>
where
    E: Evidence,
    SK: StoreKey,
    DB: Database,
{
    pub fn new(routes: HashMap<String, Handler<'a, DB, SK, E>>) -> Self {
        Self { routes }
    }

    // TODO: for now type is constrained to be constructed via single hash map. If the methods are
    // required then we should extend structure by field `sealed` and handle it in genesis
    // pub fn add_route(&mut self, route: String, handler: Handler<CTX, DB, SK, E>);
    // pub fn seal(&mut self);
    // pub fn sealed(&self) -> bool;

    pub fn has_route(&self, route: &str) -> bool {
        self.routes.contains_key(route)
    }

    pub fn route(&self, route: &str) -> Option<&Handler<'a, DB, SK, E>> {
        // in sdk checks presence and panics if route doesn't exist
        self.routes.get(route)
    }
}

impl<'a, DB, SK, E> Clone for Router<'a, DB, SK, E>
where
    E: Evidence,
    SK: StoreKey,
    DB: Database,
{
    fn clone(&self) -> Self {
        Router::new(HashMap::new())
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use gears::{
//         context::{init::InitContext, QueryableContext},
//         store::{database::MemDB, types::multi::MultiBank, ApplicationStore},
//         tendermint::types::{chain_id::ChainId, time::timestamp::Timestamp},
//     };
//     use prost::Message;
//     use std::str::FromStr;
//     use strum::EnumIter;
//
//     #[derive(Message)]
//     struct MockEvidence {}
//     impl Evidence for MockEvidence {
//         fn hash(&self) -> gears::tendermint::informal::Hash {
//             gears::tendermint::informal::Hash::from_str(
//                 "BC0E95DA6BA637BDDAADE5E6911FDB20030956A4BB8E305FB1C390FF7BCEA20",
//             )
//             .unwrap()
//         }
//         fn route(&self) -> String {
//             "mock".into()
//         }
//         fn r#type(&self) -> String {
//             "mock".into()
//         }
//         fn string(&self) -> String {
//             "".into()
//         }
//         fn height(&self) -> i64 {
//             0
//         }
//         fn validate_basic(&self) -> anyhow::Result<()> {
//             Ok(())
//         }
//     }
//
//     #[test]
//     fn create_handler() -> anyhow::Result<()> {
//         let db = MemDB::new();
//         #[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
//         pub enum StKey {
//             Foo,
//             Params,
//         }
//         impl StoreKey for StKey {
//             fn name(&self) -> &'static str {
//                 match self {
//                     StKey::Foo => "foo",
//                     StKey::Params => "params",
//                 }
//             }
//
//             fn params() -> &'static Self {
//                 const PARAM_KEY: StKey = StKey::Params;
//
//                 &PARAM_KEY
//             }
//         }
//         let mut multi_store = MultiBank::<MemDB, StKey, ApplicationStore>::new(db);
//         let mut ctx = InitContext::new(
//             &mut multi_store,
//             0,
//             Timestamp::UNIX_EPOCH,
//             ChainId::default(),
//         );
//
//         let handler_fn =
//             |ctx: &mut InitContext<'_, MemDB, StKey>, e: &MockEvidence| -> anyhow::Result<()> {
//                 // check
//                 let _kv_store = ctx.kv_store_mut(&StKey::Foo);
//                 assert_eq!(ctx.height(), e.height() as u32);
//                 Ok(())
//             };
//         let handlers_map = HashMap::from([("mock".into(), Handler::new(handler_fn.into()))]);
//         let router = Router::new(handlers_map);
//
//         assert!(router.has_route("mock"));
//         assert!(!router.has_route("foo"));
//
//         let r = router
//             .route("mock")
//             .unwrap()
//             .handle(&mut ctx, &MockEvidence {});
//         assert!(r.is_ok());
//         Ok(())
//     }
// }
