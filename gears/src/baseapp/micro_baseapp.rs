use bytes::Bytes;
use core::hash::Hash;
use database::{Database, RocksDB};
use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::tx::v1beta1::tx_v2::{self, Message, TxWithRaw};
use proto_types::AccAddress;
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};
use store_crate::{MultiStore, StoreKey};
use strum::IntoEnumIterator;
use tendermint_abci::Application;
use tendermint_informal::block::Header;
use tendermint_proto::abci::{
    RequestDeliverTx, RequestInitChain, ResponseCommit, ResponseDeliverTx, ResponseInitChain,
};
use tracing::info;

use crate::{
    error::AppError,
    types::context_v2::{Context, InitContext, TxContext},
    x::params::{Keeper, ParamsSubspaceKey},
};

use super::{
    ante_v2::{AnteHandler, AuthKeeper, BankKeeper},
    params::BaseAppParamsKeeper,
};

pub trait Handler<M: Message, SK: StoreKey>: Clone + Send + Sync {
    fn handle<DB: Database>(&self, ctx: &mut Context<DB, SK>, msg: &M) -> Result<(), AppError>;

    fn init_genesis<DB: Database>(&self, ctx: &mut Context<DB, SK>, raw: Bytes);
}

#[derive(Debug, Clone)]
pub struct MicroBaseApp<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
    AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
    H: Handler<M, SK>,
> {
    pub multi_store: Arc<RwLock<MultiStore<RocksDB, SK>>>,
    height: Arc<RwLock<u64>>,
    base_ante_handler: AnteHandler<BK, AK, SK>,
    handler: H,
    block_header: Arc<RwLock<Option<Header>>>, // passed by Tendermint in call to begin_block
    baseapp_params_keeper: BaseAppParamsKeeper<SK, PSK>,
    pub m: PhantomData<M>,
    // pub d: PhantomData<D>,
    // pub r: PhantomData<R>,
}

impl<
        M: Message + 'static,
        // D: Decoder<M> + 'static,
        // R: Router<M> + 'static,
        SK: StoreKey + Clone + Send + Sync + 'static,
        PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
        BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
        AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
        H: Handler<M, SK> + 'static,
    > Application for MicroBaseApp<SK, PSK, M, BK, AK, H>
{
    fn init_chain(&self, request: RequestInitChain) -> ResponseInitChain {
        info!("Got init chain request");
        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");

        //TODO: handle request height > 1 as is done in SDK

        let mut ctx = InitContext::new(&mut multi_store, self.get_block_height(), request.chain_id);

        if let Some(params) = request.consensus_params.clone() {
            self.baseapp_params_keeper
                .set_consensus_params(&mut ctx.as_any(), params);
        }

        self.handler
            .init_genesis(&mut ctx.as_any(), request.app_state_bytes);

        multi_store.write_then_clear_tx_caches();

        ResponseInitChain {
            consensus_params: request.consensus_params,
            validators: request.validators,
            app_hash: "hash_goes_here".into(),
        }
    }

    fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx {
        info!("Got deliver tx request");
        match self.run_tx(request.tx) {
            Ok(events) => ResponseDeliverTx {
                code: 0,
                data: Default::default(),
                log: "".to_string(),
                info: "".to_string(),
                gas_wanted: 0,
                gas_used: 0,
                events: events.into_iter().map(|e| e.into()).collect(),
                codespace: "".to_string(),
            },
            Err(e) => {
                info!("Failed to process tx: {}", e);
                ResponseDeliverTx {
                    code: e.code(),
                    data: Bytes::new(),
                    log: e.to_string(),
                    info: "".to_string(),
                    gas_wanted: 0,
                    gas_used: 0,
                    events: vec![],
                    codespace: "".to_string(),
                }
            }
        }

        // info!("Got deliver tx request");

        // let mut multi_store = self
        //     .multi_store
        //     .write()
        //     .expect("RwLock will not be poisoned");
        // let mut ctx = TxContext::new(
        //     &mut multi_store,
        //     self.get_block_height(),
        //     self.get_block_header()
        //         .expect("block header is set in begin block"),
        //     request.tx.clone().into(),
        // );

        // let tx: tx_v2::Tx<M> = tx_v2::Tx::decode(request.tx).unwrap();

        // let msgs = tx.get_msgs();

        // for msg in msgs {
        //     self.handler.handle(&mut ctx.as_any(), msg);
        // }

        // match Self::run_msgs(&mut ctx.as_any(), tx.get_msgs()) {
        //     Ok(_) => {
        //         let events = ctx.events;
        //         multi_store.write_then_clear_tx_caches();
        //         Ok(events)
        //     }
        //     Err(e) => {
        //         multi_store.clear_tx_caches();
        //         Err(e)
        //     }
        // }

        //self.base_ante_handler.run(ctx, tx);

        // let bank_key = S::get_bank_key();

        // let multi_store = self
        //     .multi_store
        //     .read()
        //     .expect("RwLock will not be poisoned");

        // let bank_store = multi_store.get_kv_store(&bank_key);

        // let signers = msg.get_signers();

        // R::route_msg(msg);
        //ResponseDeliverTx::default()
    }

    fn commit(&self) -> ResponseCommit {
        info!("Got commit request");
        let new_height = self.increment_block_height();
        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");

        let hash = multi_store.commit();
        info!(
            "Committed state, block height: {} app hash: {}",
            new_height,
            hex::encode(hash)
        );

        ResponseCommit {
            data: hash.to_vec().into(),
            retain_height: (new_height - 1)
                .try_into()
                .expect("can't believe we made it this far"),
        }
    }
}

impl<
        M: Message,
        // D: Decoder<M>,
        // R: Router<M>,
        SK: StoreKey,
        PSK: ParamsSubspaceKey + Clone + Send + Sync + 'static,
        BK: BankKeeper<SK> + Clone + Send + Sync + 'static,
        AK: AuthKeeper<SK> + Clone + Send + Sync + 'static,
        H: Handler<M, SK>,
    > MicroBaseApp<SK, PSK, M, BK, AK, H>
{
    pub fn new(
        db: RocksDB,
        app_name: &'static str,
        bank_keeper: BK,
        auth_keeper: AK,
        params_keeper: Keeper<SK, PSK>,
        params_subspace_key: PSK,
        handler: H,
    ) -> Self {
        let multi_store = MultiStore::new(db);
        let baseapp_params_keeper = BaseAppParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        let height = multi_store.get_head_version().into();
        Self {
            multi_store: Arc::new(RwLock::new(multi_store)),
            base_ante_handler: AnteHandler::new(bank_keeper, auth_keeper),
            handler,
            block_header: Arc::new(RwLock::new(None)),
            baseapp_params_keeper,
            height: Arc::new(RwLock::new(height)),
            //block_header: Arc::new(RwLock::new(None)),
            //app_name,
            m: PhantomData,
            // d: PhantomData,
            // r: PhantomData,
        }
    }

    pub fn get_block_height(&self) -> u64 {
        *self.height.read().expect("RwLock will not be poisoned")
    }

    fn get_block_header(&self) -> Option<Header> {
        self.block_header
            .read()
            .expect("RwLock will not be poisoned")
            .clone()
    }

    fn run_tx(&self, raw: Bytes) -> Result<Vec<tendermint_informal::abci::Event>, AppError> {
        // let tx: tx_v2::Tx<M> =
        //     tx_v2::Tx::decode(raw.clone()).map_err(|e| AppError::TxParseError(e.to_string()))?;

        let tx_with_raw: TxWithRaw<M> = TxWithRaw::from_bytes(raw.clone())
            .map_err(|e| AppError::TxParseError(e.to_string()))?;

        Self::validate_basic_tx_msgs(tx_with_raw.tx.get_msgs())?;

        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");

        let mut ctx = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block"),
            raw.clone().into(),
        );

        match self.base_ante_handler.run(&mut ctx.as_any(), &tx_with_raw) {
            Ok(_) => multi_store.write_then_clear_tx_caches(),
            Err(e) => {
                multi_store.clear_tx_caches();
                return Err(e);
            }
        };

        // match AnteHandler::run(&mut ctx.as_any(), &tx) {
        //     Ok(_) => multi_store.write_then_clear_tx_caches(),
        //     Err(e) => {
        //         multi_store.clear_tx_caches();
        //         return Err(e);
        //     }
        // };

        let mut ctx = TxContext::new(
            &mut multi_store,
            self.get_block_height(),
            self.get_block_header()
                .expect("block header is set in begin block"),
            raw.into(),
        );

        match self.run_msgs(&mut ctx.as_any(), tx_with_raw.tx.get_msgs()) {
            Ok(_) => {
                let events = ctx.events;
                multi_store.write_then_clear_tx_caches();
                Ok(events)
            }
            Err(e) => {
                multi_store.clear_tx_caches();
                Err(e)
            }
        }
    }

    fn run_msgs<T: Database>(
        &self,
        ctx: &mut Context<T, SK>,
        msgs: &Vec<M>,
    ) -> Result<(), AppError> {
        for msg in msgs {
            self.handler.handle(ctx, msg)?
        }

        return Ok(());
    }

    fn increment_block_height(&self) -> u64 {
        let mut height = self.height.write().expect("RwLock will not be poisoned");
        *height += 1;
        return *height;
    }

    fn validate_basic_tx_msgs(msgs: &Vec<M>) -> Result<(), AppError> {
        if msgs.is_empty() {
            return Err(AppError::InvalidRequest(
                "must contain at least one message".into(),
            ));
        }

        for msg in msgs {
            msg.validate_basic()
                .map_err(|e| AppError::TxValidation(e.to_string()))?
        }

        return Ok(());
    }

    //fn run_tx(&self, raw: Bytes) -> Result<Vec<tendermint_informal::abci::Event>, AppError> {}
    // TODO:
    // 1. Check from address is signer + verify signature

    //###########################
    // let tx = Tx::decode(raw.clone()).unwrap();

    // let public = tx.auth_info.clone().unwrap().signer_infos[0]
    //     .clone()
    //     .public_key
    //     .unwrap()
    //     .type_url;
    // println!("################# URL:  {}", public);
    // //cosmos.crypto.secp256k1.PubKey
    // // let msgs = tx.get_msgs();
    // // let msg = &msgs[0];

    // // let signers = msg.get_signers();

    // // println!("################### Signers: {}", signers);

    // // Ok(())

    //     //#######################
    //     let tx = DecodedTx::from_bytes(raw.clone())?;

    //     Self::validate_basic_tx_msgs(tx.get_msgs())?;

    //     let mut multi_store = self
    //         .multi_store
    //         .write()
    //         .expect("RwLock will not be poisoned");
    //     let mut ctx = TxContext::new(
    //         &mut multi_store,
    //         self.get_block_height(),
    //         self.get_block_header()
    //             .expect("block header is set in begin block"),
    //         raw.clone().into(),
    //     );

    //     match AnteHandler::run(&mut ctx.as_any(), &tx) {
    //         Ok(_) => multi_store.write_then_clear_tx_caches(),
    //         Err(e) => {
    //             multi_store.clear_tx_caches();
    //             return Err(e);
    //         }
    //     };

    //     let mut ctx = TxContext::new(
    //         &mut multi_store,
    //         self.get_block_height(),
    //         self.get_block_header()
    //             .expect("block header is set in begin block"),
    //         raw.into(),
    //     );

    //     match Self::run_msgs(&mut ctx.as_any(), tx.get_msgs()) {
    //         Ok(_) => {
    //             let events = ctx.events;
    //             multi_store.write_then_clear_tx_caches();
    //             Ok(events)
    //         }
    //         Err(e) => {
    //             multi_store.clear_tx_caches();
    //             Err(e)
    //         }
    //     }
    // }

    // fn run_msgs<T: DB>(ctx: &mut Context<T>, msgs: &Vec<Msg>) -> Result<(), AppError> {
    //     for msg in msgs {
    //         match msg {
    //             Msg::Send(send_msg) => {
    //                 Bank::send_coins_from_account_to_account(ctx, send_msg.clone())?
    //             }
    //         };
    //     }

    //     return Ok(());
    // }

    // fn validate_basic_tx_msgs(msgs: &Vec<Msg>) -> Result<(), AppError> {
    //     if msgs.is_empty() {
    //         return Err(AppError::InvalidRequest(
    //             "must contain at least one message".into(),
    //         ));
    //     }

    //     for msg in msgs {
    //         msg.validate_basic()
    //             .map_err(|e| AppError::TxValidation(e.to_string()))?
    //     }

    //     return Ok(());
    // }
    // pub fn deliver_tx<M: Message, D: Decoder<M>, R: Router<M>>(
    //     &self,
    //     raw: Vec<u8>,
    // ) -> Result<(), String> {
    //     let msg = D::decode(raw);

    //     let bank_key = S::get_bank_key();

    //     let multi_store = self
    //         .multi_store
    //         .read()
    //         .expect("RwLock will not be poisoned");

    //     let bank_store = multi_store.get_kv_store(&bank_key);

    //     let signers = msg.get_signers();

    //     R::route_msg(msg);
    //     Ok(())
    // }
}
