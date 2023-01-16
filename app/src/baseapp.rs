use std::sync::{Arc, RwLock};

use ibc_proto::cosmos::{
    auth::v1beta1::{BaseAccount, QueryAccountRequest},
    bank::v1beta1::MsgSend,
    base::v1beta1::Coin,
    tx::v1beta1::{Tx, TxBody},
};
use prost::Message;

use bytes::Bytes;
use tendermint_abci::Application;
use tendermint_proto::abci::{
    Event, EventAttribute, RequestCheckTx, RequestDeliverTx, RequestInfo, RequestQuery,
    ResponseCheckTx, ResponseCommit, ResponseDeliverTx, ResponseInfo, ResponseQuery,
};
use tracing::{debug, info};

use crate::{
    crypto::verify_signature,
    store::MultiStore,
    types::{AccAddress, Context},
    x::{
        auth::Auth,
        bank::{Balance, Bank, GenesisState},
    },
};

pub const BANK_STORE_PREFIX: [u8; 4] = [098, 097, 110, 107]; // "bank"
pub const AUTH_STORE_PREFIX: [u8; 3] = [097, 099, 099]; // "acc" - use acc even though it's the auth store to match cosmos SDK

#[derive(Debug, Clone)]
pub struct BaseApp {
    multi_store: Arc<RwLock<MultiStore>>,
    height: Arc<RwLock<u32>>,
}

impl BaseApp {
    pub fn new() -> Self {
        let store = MultiStore::new();

        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32(
                    &"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
                )
                .expect("this won't fail"),
                coins: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: cosmwasm_std::Uint256::from(34_u32),
                }],
            }],
        };

        let mut ctx = Context::new(store);
        Bank::init_genesis(&mut ctx, genesis);

        let genesis = crate::x::auth::GenesisState {
            accounts: vec![BaseAccount {
                address: "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into(),
                pub_key: None,
                account_number: 0,
                sequence: 0,
            }],
        };
        Auth::init_genesis(&mut ctx, genesis).expect("genesis is valid");

        Self {
            multi_store: Arc::new(RwLock::new(ctx.multi_store)),
            height: Arc::new(RwLock::new(0)),
        }
    }

    fn get_block_height(&self) -> u32 {
        *self.height.read().expect("RwLock will not be poisoned")
    }

    fn increment_block_height(&self) -> u32 {
        let mut height = self.height.write().expect("RwLock will not be poisoned");
        *height += 1;
        return *height;
    }
}

impl Application for BaseApp {
    fn info(&self, request: RequestInfo) -> ResponseInfo {
        debug!(
            "Got info request. Tendermint version: {}; Block version: {}; P2P version: {}",
            request.version, request.block_version, request.p2p_version
        );

        ResponseInfo {
            data: "gaia-rs".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height: self.get_block_height().into(),
            last_block_app_hash: "hash_goes_here".into(),
        }
    }

    fn query(&self, request: RequestQuery) -> ResponseQuery {
        info!("Handling query to: {}", request.path);

        match request.path.as_str() {
            "/cosmos.bank.v1beta1.Query/AllBalances" => {
                let data = request.data.clone();
                let req = ibc_proto::cosmos::bank::v1beta1::QueryAllBalancesRequest::decode(data)
                    .unwrap();

                let store = self.multi_store.read().unwrap();
                let ctx = Context::new(store.clone());

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
                            height: self.get_block_height().into(),
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
                        height: self.get_block_height().into(),
                        codespace: "".to_string(),
                    },
                }
            }
            "/cosmos.auth.v1beta1.Query/Account" => {
                let data = request.data.clone();
                let req = QueryAccountRequest::decode(data).unwrap();

                let store = self.multi_store.read().unwrap();
                let ctx = Context::new(store.clone());

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
                        code: 0,
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
                code: 0,
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
        // TODO:
        // 1. Update account sequence etc - should this be done externally?
        // 2. Remove unwraps
        // 4. Check from address is signer
        // 5. Handle Tx fees
        // 6. Handle app errors e.g. what happens when insufficient funds?

        //###########################
        let tx = Tx::decode(request.tx.clone()).unwrap();
        let body = tx.body.unwrap();
        let url = body.messages[0].clone().type_url;
        info!("Handling deliver tx to: {}", url);

        match url.as_str() {
            "/cosmos.bank.v1beta1.MsgSend" => {
                let tx_raw =
                    ibc_proto::cosmos::tx::v1beta1::TxRaw::decode(request.tx.clone()).unwrap();
                let tx = Tx::decode(request.tx).unwrap();
                verify_signature(tx.clone(), tx_raw);

                let body = tx.body.unwrap();

                let url = body.messages[0].clone().type_url;

                let request =
                    MsgSend::decode::<Bytes>(body.messages[0].clone().value.into()).unwrap();

                let mut multi_store = self.multi_store.write().unwrap();
                let transient_store = multi_store.clone();
                let mut ctx = Context::new(transient_store);

                match Bank::send_coins(&mut ctx, request) {
                    Ok(_) => *multi_store = ctx.multi_store,
                    Err(_) => (),
                }

                ResponseDeliverTx {
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
                }
            }
            _ => {
                dbg!("Rejecting tx: no such message type");
                ResponseDeliverTx {
                    code: 6,
                    data: "".into(),
                    log: "No such message type".into(),
                    info: "".into(),
                    gas_wanted: 0,
                    gas_used: 0,
                    events: vec![],
                    codespace: "".into(),
                }
            }
        }
    }

    fn commit(&self) -> ResponseCommit {
        let new_height = self.increment_block_height();

        ResponseCommit {
            data: "hash_goes_here".into(),
            retain_height: (new_height - 1).into(),
        }
    }
}
