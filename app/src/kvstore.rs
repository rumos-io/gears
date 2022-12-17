use std::{
    collections::HashMap,
    str::FromStr,
    sync::mpsc::{channel, Receiver, Sender},
};

use ibc_proto::{
    cosmos::{auth::v1beta1::BaseAccount, bank::v1beta1::MsgSend, base::v1beta1::Coin},
    google::protobuf::Any,
};
use prost::Message;

use bytes::{Bytes, BytesMut};
use tendermint_abci::{Application, Error};
use tendermint_proto::abci::{
    Event, EventAttribute, RequestCheckTx, RequestDeliverTx, RequestInfo, RequestQuery,
    ResponseCheckTx, ResponseCommit, ResponseDeliverTx, ResponseInfo, ResponseQuery,
};
use tracing::{debug, info};

use crate::{
    bank::{Address, Balance, Bank, GenesisState},
    store::Store,
};

pub const MAX_VARINT_LENGTH: usize = 16; //TODO: fix this

#[derive(Debug, Clone)]
pub struct KeyValueStoreApp {
    cmd_tx: Sender<Command>,
    bank: Bank,
}

impl KeyValueStoreApp {
    pub fn new() -> (Self, KeyValueStoreDriver) {
        let (cmd_tx, cmd_rx) = channel();
        let store = Store::new();
        let genesis = GenesisState {
            balances: vec![Balance {
                address: Address::new("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string())
                    .unwrap(),
                coins: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: cosmwasm_std::Uint256::from(34_u32),
                }],
            }],
        };
        let bank = Bank::new(store, genesis);
        (Self { cmd_tx, bank }, KeyValueStoreDriver::new(cmd_rx))
    }

    /// Attempt to retrieve the value associated with the given key.
    pub fn get(&self, key: Vec<u8>) -> Result<(i64, Option<Vec<u8>>), Error> {
        let (result_tx, result_rx) = channel();
        channel_send(
            &self.cmd_tx,
            Command::Get {
                key: key,
                result_tx,
            },
        )?;
        channel_recv(&result_rx)
    }

    /// Attempt to set the value associated with the given key.
    ///
    /// Optionally returns any pre-existing value associated with the given
    /// key.
    pub fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let (result_tx, result_rx) = channel();
        channel_send(
            &self.cmd_tx,
            Command::Set {
                key: key,
                value: value,
                result_tx,
            },
        )?;
        channel_recv(&result_rx)
    }
}

impl Application for KeyValueStoreApp {
    fn info(&self, request: RequestInfo) -> ResponseInfo {
        debug!(
            "Got info request. Tendermint version: {}; Block version: {}; P2P version: {}",
            request.version, request.block_version, request.p2p_version
        );

        let (result_tx, result_rx) = channel();
        channel_send(&self.cmd_tx, Command::GetInfo { result_tx }).unwrap();
        let (last_block_height, last_block_app_hash) = channel_recv(&result_rx).unwrap();

        ResponseInfo {
            data: "kvstore-rs".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height,
            last_block_app_hash: last_block_app_hash.into(),
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
                    height: 1, //TODO: get actual height
                    codespace: "".to_string(),
                }

                // match self.get(req.address.clone().into()) {
                //     Ok((height, value_opt)) => match value_opt {
                //         Some(value) => {
                //             let res = ibc_proto::cosmos::bank::v1beta1::QueryAllBalancesResponse {
                //                 balances: vec![ibc_proto::cosmos::base::v1beta1::Coin {
                //                     denom: "uatom".into(),
                //                     amount: cosmwasm_std::Uint256::from_str(
                //                         &String::from_utf8(value).unwrap(),
                //                     )
                //                     .unwrap(),
                //                 }],
                //                 pagination: None,
                //             };

                //             let res = res.encode_to_vec();

                //             ResponseQuery {
                //                 code: 0,
                //                 log: "exists".to_string(),
                //                 info: "".to_string(),
                //                 index: 0,
                //                 key: request.data,
                //                 value: res.into(),
                //                 proof_ops: None,
                //                 height,
                //                 codespace: "".to_string(),
                //             }
                //         }
                //         None => ResponseQuery {
                //             code: 0,
                //             log: "address does not exist".to_string(),
                //             info: "".to_string(),
                //             index: 0,
                //             key: request.data,
                //             value: Default::default(),
                //             proof_ops: None,
                //             height,
                //             codespace: "".to_string(),
                //         },
                //     },
                //     Err(e) => panic!("Failed to get key \"{}\": {:?}", req.address, e),
                // }
            }
            "/cosmos.auth.v1beta1.Query/Account" => {
                let data = request.data.clone();

                let req =
                    ibc_proto::cosmos::auth::v1beta1::QueryAccountRequest::decode(data).unwrap();

                let account = BaseAccount {
                    address: req.address,
                    pub_key: None,
                    account_number: 1,
                    sequence: 1,
                };

                let res = ibc_proto::cosmos::auth::v1beta1::QueryAccountResponse {
                    account: Some(Any {
                        type_url: "/cosmos.auth.v1beta1.BaseAccount".to_string(),
                        value: account.encode_to_vec(),
                    }),
                };

                let res = res.encode_to_vec();

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
        let tx = ibc_proto::cosmos::tx::v1beta1::Tx::decode(request.tx).unwrap();

        let body = tx.body.unwrap();

        let msg: MsgSend = ibc_proto::cosmos::bank::v1beta1::MsgSend::decode::<Bytes>(
            body.messages[0].clone().value.into(),
        )
        .unwrap();

        debug!("msg from address {}", msg.from_address);
        debug!("msg from address {}", msg.to_address);
        debug!("msg from address {:?}", msg.amount);

        ResponseDeliverTx {
            code: 0,
            data: Default::default(),
            log: "".to_string(),
            info: "".to_string(),
            gas_wanted: 0,
            gas_used: 0,
            events: vec![Event {
                r#type: "app".to_string(),
                attributes: vec![
                    EventAttribute {
                        key: "key".into(),
                        value: "nothing".into(),
                        index: true,
                    },
                    EventAttribute {
                        key: "index_key".into(),
                        value: "index is working".into(),
                        index: true,
                    },
                    EventAttribute {
                        key: "noindex_key".into(),
                        value: "index is working".into(),
                        index: false,
                    },
                ],
            }],
            codespace: "".to_string(),
        }
    }

    fn commit(&self) -> ResponseCommit {
        let (result_tx, result_rx) = channel();
        channel_send(&self.cmd_tx, Command::Commit { result_tx }).unwrap();
        let (height, app_hash) = channel_recv(&result_rx).unwrap();
        info!("Committed height {}", height);
        ResponseCommit {
            data: app_hash.into(),
            retain_height: height - 1,
        }
    }
}

/// Manages key/value store state.
#[derive(Debug)]
pub struct KeyValueStoreDriver {
    store: HashMap<Vec<u8>, Vec<u8>>,
    height: i64,
    app_hash: Vec<u8>,
    cmd_rx: Receiver<Command>,
}

impl KeyValueStoreDriver {
    fn new(cmd_rx: Receiver<Command>) -> Self {
        let mut store = HashMap::new();

        // Initialize a hard coded genesis account
        let key = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".into();
        let value: Vec<u8> = cosmwasm_std::Uint256::from(34_u32).to_string().into();
        store.insert(key, value);

        Self {
            store,
            height: 0,
            app_hash: vec![0_u8; MAX_VARINT_LENGTH],
            cmd_rx,
        }
    }

    /// Run the driver in the current thread (blocking).
    pub fn run(mut self) -> Result<(), Error> {
        loop {
            let cmd = self.cmd_rx.recv().map_err(Error::channel_recv)?;
            match cmd {
                Command::GetInfo { result_tx } => {
                    channel_send(&result_tx, (self.height, self.app_hash.clone()))?
                }
                Command::Get { key, result_tx } => {
                    debug!("Getting value for \"{:?}\"", key);
                    channel_send(
                        &result_tx,
                        (self.height, self.store.get(&key).map(Clone::clone)),
                    )?;
                }
                Command::Set {
                    key,
                    value,
                    result_tx,
                } => {
                    debug!("Setting \"{:?}\" = \"{:?}\"", key, value);
                    channel_send(&result_tx, self.store.insert(key, value))?;
                }
                Command::Commit { result_tx } => self.commit(result_tx)?,
            }
        }
    }

    fn commit(&mut self, result_tx: Sender<(i64, Vec<u8>)>) -> Result<(), Error> {
        // As in the Go-based key/value store, simply encode the number of
        // items as the "app hash"
        let mut app_hash = BytesMut::with_capacity(MAX_VARINT_LENGTH);
        prost::encoding::encode_varint(self.store.len() as u64, &mut app_hash);
        self.app_hash = app_hash.to_vec();
        self.height += 1;
        channel_send(&result_tx, (self.height, self.app_hash.clone()))
    }
}

#[derive(Debug, Clone)]
enum Command {
    /// Get the height of the last commit.
    GetInfo { result_tx: Sender<(i64, Vec<u8>)> },
    /// Get the key associated with `key`.
    Get {
        key: Vec<u8>,
        result_tx: Sender<(i64, Option<Vec<u8>>)>,
    },
    /// Set the value of `key` to to `value`.
    Set {
        key: Vec<u8>,
        value: Vec<u8>,
        result_tx: Sender<Option<Vec<u8>>>,
    },
    /// Commit the current state of the application, which involves recomputing
    /// the application's hash.
    Commit { result_tx: Sender<(i64, Vec<u8>)> },
}

fn channel_send<T>(tx: &Sender<T>, value: T) -> Result<(), Error> {
    tx.send(value).map_err(Error::send)
}

fn channel_recv<T>(rx: &Receiver<T>) -> Result<T, Error> {
    rx.recv().map_err(Error::channel_recv)
}
