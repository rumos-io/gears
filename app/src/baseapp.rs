use std::sync::{Arc, RwLock};

use ibc_proto::cosmos::{
    auth::v1beta1::QueryAccountRequest,
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
use tracing::debug;

use crate::{
    crypto::verify_signature,
    store::Store,
    types::AccAddress,
    x::{
        auth::Auth,
        bank::{Balance, Bank, GenesisState},
    },
};

const BANK_STORE_PREFIX: [u8; 1] = [2];
const AUTH_STORE_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct BaseApp {
    bank: Bank,
    auth: Auth,
    height: Arc<RwLock<u32>>,
}

impl BaseApp {
    pub fn new() -> Self {
        let store = Store::new();
        let bank_store = store.get_sub_store(BANK_STORE_PREFIX.into());
        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32(
                    &"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
                )
                .unwrap(),
                coins: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: cosmwasm_std::Uint256::from(34_u32),
                }],
            }],
        };
        let bank = Bank::new(bank_store, genesis);

        let auth_store = store.get_sub_store(AUTH_STORE_PREFIX.into());
        let auth = Auth::new(auth_store);
        Self {
            auth,
            bank,
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
        debug!("Processing query. Path: {}", request.path);

        match request.path.as_str() {
            "/cosmos.bank.v1beta1.Query/AllBalances" => {
                let data = request.data.clone();
                let req = ibc_proto::cosmos::bank::v1beta1::QueryAllBalancesRequest::decode(data)
                    .unwrap();

                let res = self.bank.query_all_balances(req);
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
            "/cosmos.auth.v1beta1.Query/Account" => {
                let data = request.data.clone();
                let req = QueryAccountRequest::decode(data).unwrap();

                let res = self.auth.query_account(req).encode_to_vec();

                ResponseQuery {
                    code: 0,
                    log: "exists".to_string(),
                    info: "".to_string(),
                    index: 0,
                    key: request.data,
                    value: res.into(),
                    proof_ops: None,
                    height: 0,
                    codespace: "".to_string(),
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
        // 1. update account sequence etc - should this be done externally?

        let tx_raw = ibc_proto::cosmos::tx::v1beta1::TxRaw::decode(request.tx.clone()).unwrap();
        let tx = Tx::decode(request.tx).unwrap();
        verify_signature(tx.clone(), tx_raw);

        let body = tx.body.unwrap();

        let url = body.messages[0].clone().type_url;

        // println!("URL: {}", url);
        // /cosmos.bank.v1beta1.MsgSend
        let request = MsgSend::decode::<Bytes>(body.messages[0].clone().value.into()).unwrap();

        self.bank.send_coins(request);

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

    fn commit(&self) -> ResponseCommit {
        let new_height = self.increment_block_height();

        ResponseCommit {
            data: "hash_goes_here".into(),
            retain_height: (new_height - 1).into(),
        }
    }
}
