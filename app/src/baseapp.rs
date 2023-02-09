use std::sync::{Arc, RwLock};

use ibc_proto::cosmos::{
    base::v1beta1::Coin,
    tx::v1beta1::{Tx, TxBody},
};
use prost::Message;

use bytes::Bytes;
use proto_types::AccAddress;
use tendermint_abci::Application;
use tendermint_proto::abci::{
    Event, EventAttribute, RequestCheckTx, RequestDeliverTx, RequestInfo, RequestInitChain,
    RequestQuery, ResponseCheckTx, ResponseCommit, ResponseDeliverTx, ResponseInfo,
    ResponseInitChain, ResponseQuery,
};
use tracing::{debug, info};

use crate::{
    ante::AnteHandler,
    crypto::verify_signature,
    error::AppError,
    store::MultiStore,
    types::{
        proto::{BaseAccount, QueryAccountRequest, QueryAllBalancesRequest},
        Context, DecodedTx, Msg,
    },
    x::{
        auth::{Auth, DEFAULT_PARAMS},
        bank::{Balance, Bank, GenesisState},
    },
};

//TODO:
// 1. Remove unwraps

#[derive(Debug, Clone)]
pub struct BaseApp {
    multi_store: Arc<RwLock<MultiStore>>,
    height: Arc<RwLock<u64>>,
}

impl BaseApp {
    pub fn new() -> Self {
        Self {
            multi_store: Arc::new(RwLock::new(MultiStore::new())),
            height: Arc::new(RwLock::new(0)),
        }
    }

    fn get_block_height(&self) -> u64 {
        *self.height.read().expect("RwLock will not be poisoned")
    }

    fn increment_block_height(&self) -> u64 {
        let mut height = self.height.write().expect("RwLock will not be poisoned");
        *height += 1;
        return *height;
    }

    fn run_tx(&self, raw: Bytes) -> Result<(), AppError> {
        // TODO:
        // 1. Update account sequence etc - should this be done externally?
        // 2. Check from address is signer + verify signature
        // 3. Handle Tx fees

        //###########################
        let tx = Tx::decode(raw.clone())?;

        let public = tx.auth_info.clone().unwrap().signer_infos[0]
            .clone()
            .public_key
            .unwrap()
            .type_url;
        println!("################# URL:  {}", public);
        ///cosmos.crypto.secp256k1.PubKey
        // let msgs = tx.get_msgs();
        // let msg = &msgs[0];

        // let signers = msg.get_signers();

        // println!("################### Signers: {}", signers);

        // Ok(())

        //#######################
        let tx = DecodedTx::from_bytes(raw)?;

        BaseApp::validate_basic_tx_msgs(tx.get_msgs())?;

        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");
        let transient_store = multi_store.clone();
        let mut ctx = Context::new(transient_store, self.get_block_height());

        AnteHandler::run(&mut ctx, &tx)?;
        *multi_store = ctx.multi_store.clone();

        BaseApp::run_msgs(&mut ctx, tx.get_msgs())?;
        *multi_store = ctx.multi_store;

        Ok(())
    }

    fn run_msgs(ctx: &mut Context, msgs: &Vec<Msg>) -> Result<(), AppError> {
        for msg in msgs {
            match msg {
                Msg::Send(send_msg) => {
                    Bank::send_coins_from_account_to_account(ctx, send_msg.clone())?
                }
                Msg::Test => return Err(AppError::AccountNotFound),
            };
        }

        return Ok(());
    }

    fn validate_basic_tx_msgs(msgs: &Vec<Msg>) -> Result<(), AppError> {
        if msgs.is_empty() {
            return Err(AppError::InvalidRequest(
                "must contain at least one message".into(),
            ));
        }

        for msg in msgs {
            msg.validate_basic()?
        }

        return Ok(());
    }
}

impl Application for BaseApp {
    fn init_chain(&self, request: RequestInitChain) -> ResponseInitChain {
        let mut multi_store = self
            .multi_store
            .write()
            .expect("RwLock will not be poisoned");
        let transient_store = multi_store.clone();
        let mut ctx = Context::new(transient_store, self.get_block_height());

        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32(
                    &"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
                )
                .expect("hard coded address is valid"),
                coins: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: cosmwasm_std::Uint256::from(34_u32),
                }],
            }],
        };
        Bank::init_genesis(&mut ctx, genesis);

        let genesis = crate::x::auth::GenesisState {
            accounts: vec![BaseAccount {
                address: AccAddress::from_bech32(
                    "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into(),
                )
                .expect("hard coded address is valid"),
                pub_key: None,
                account_number: 0,
                sequence: 0,
            }],
            params: DEFAULT_PARAMS,
        };
        Auth::init_genesis(&mut ctx, genesis).expect("genesis is valid");

        *multi_store = ctx.multi_store;

        ResponseInitChain {
            consensus_params: request.consensus_params,
            validators: request.validators,
            app_hash: "hash_goes_here".into(),
        }
    }

    fn info(&self, request: RequestInfo) -> ResponseInfo {
        debug!(
            "Got info request. Tendermint version: {}; Block version: {}; P2P version: {}",
            request.version, request.block_version, request.p2p_version
        );

        ResponseInfo {
            data: "gaia-rs".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height: self
                .get_block_height()
                .try_into()
                .expect("can't believe we made it this far"),
            last_block_app_hash: "hash_goes_here".into(),
        }
    }

    fn query(&self, request: RequestQuery) -> ResponseQuery {
        info!("Handling query to: {}", request.path);

        match request.path.as_str() {
            "/cosmos.bank.v1beta1.Query/AllBalances" => {
                let data = request.data.clone();
                let req = QueryAllBalancesRequest::decode(data).unwrap();

                let store = self.multi_store.read().unwrap();
                let ctx = Context::new(store.clone(), self.get_block_height());

                let res = Bank::query_all_balances(&ctx, req);

                match res {
                    Ok(res) => {
                        let res = res.encode_to_vec();

                        ResponseQuery {
                            code: 0,
                            log: "exists".to_string(),
                            info: "".to_string(),
                            index: 0,
                            key: request.data,
                            value: res.into(),
                            proof_ops: None,
                            height: self
                                .get_block_height()
                                .try_into()
                                .expect("can't believe we made it this far"),
                            codespace: "".to_string(),
                        }
                    }
                    Err(e) => ResponseQuery {
                        code: 0,
                        log: e.to_string(),
                        info: "".to_string(),
                        index: 0,
                        key: request.data,
                        value: vec![].into(),
                        proof_ops: None,
                        height: self
                            .get_block_height()
                            .try_into()
                            .expect("can't believe we made it this far"),
                        codespace: "".to_string(),
                    },
                }
            }
            "/cosmos.auth.v1beta1.Query/Account" => {
                let data = request.data.clone();
                let req = QueryAccountRequest::decode(data).unwrap();

                let store = self.multi_store.read().unwrap();
                let ctx = Context::new(store.clone(), self.get_block_height());

                let res = Auth::query_account(&ctx, req);

                match res {
                    Ok(res) => ResponseQuery {
                        code: 0,
                        log: "exists".to_string(),
                        info: "".to_string(),
                        index: 0,
                        key: request.data,
                        value: res.encode_to_vec().into(),
                        proof_ops: None,
                        height: 0,
                        codespace: "".to_string(),
                    },
                    Err(e) => ResponseQuery {
                        code: 1,
                        log: e.to_string(),
                        info: "".to_string(),
                        index: 0,
                        key: request.data,
                        value: vec![].into(),
                        proof_ops: None,
                        height: 0,
                        codespace: "".to_string(),
                    },
                }
            }
            _ => ResponseQuery {
                code: 1,
                log: "unrecognized query".to_string(),
                info: "".to_string(),
                index: 0,
                key: request.data,
                value: Default::default(),
                proof_ops: None,
                height: 0,
                codespace: "".to_string(),
            },
        }
    }

    fn check_tx(&self, _request: RequestCheckTx) -> ResponseCheckTx {
        ResponseCheckTx {
            code: 0,
            data: Default::default(),
            log: "".to_string(),
            info: "".to_string(),
            gas_wanted: 1,
            gas_used: 0,
            events: vec![],
            codespace: "".to_string(),
            mempool_error: "".to_string(),
            priority: 0,
            sender: "".to_string(),
        }
    }

    fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx {
        match self.run_tx(request.tx) {
            Ok(_) => ResponseDeliverTx {
                code: 0,
                data: Default::default(),
                log: "".to_string(),
                info: "".to_string(),
                gas_wanted: 0,
                gas_used: 0,
                events: vec![Event {
                    r#type: "app".to_string(),
                    attributes: vec![EventAttribute {
                        key: "key".into(),
                        value: "nothing".into(),
                        index: true,
                    }],
                }],
                codespace: "".to_string(),
            },
            Err(e) => {
                info!("failed to process tx: {}", e);
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
    }

    fn commit(&self) -> ResponseCommit {
        let new_height = self.increment_block_height();

        ResponseCommit {
            data: "hash_goes_here".into(),
            retain_height: (new_height - 1)
                .try_into()
                .expect("can't believe we made it this far"),
        }
    }
}
