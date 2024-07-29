use crate::query::{
    QueryAccountRequest, QueryAccountResponse, QueryAccountsRequest, QueryAccountsResponse,
    QueryGetTxRequest, QueryGetTxsEventRequest, QueryParamsRequest, QueryParamsResponse,
    QueryTxResponse, QueryTxsResponse,
};
use anyhow::anyhow;
use bytes::Bytes;
use clap::{Args, Subcommand};
use gears::baseapp::QueryResponse;
use gears::core::errors::CoreError;
use gears::core::Protobuf;
use gears::derive::Query;
use gears::tendermint::informal::Hash;
use gears::tendermint::rpc::client::{Client, HttpClient};
use gears::tendermint::rpc::response::{
    block::Response as BlockResponse, tx::Response as CosmosTxResponse,
};
use gears::tendermint::types::proto::Protobuf as _;
use gears::types::address::AccAddress;
use gears::types::pagination::request::PaginationRequest;
use gears::types::response::tx::TxResponse;
use gears::types::tx::TxMessage;
use gears::{application::handlers::client::QueryHandler, cli::pagination::CliPaginationRequest};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

#[derive(Args, Debug)]
pub struct AuthQueryCli {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    Account(AccountCommand),
    Accounts(AccountsCommand),
    Params,
}

/// Query for account by address
#[derive(Args, Debug, Clone)]
pub struct AccountCommand {
    /// address
    pub address: AccAddress,
}

/// Query all the accounts
#[derive(Args, Debug, Clone)]
pub struct AccountsCommand {
    #[command(flatten)]
    pub pagination: CliPaginationRequest,
}

#[derive(Clone, PartialEq, Query)]
#[query(kind = "request")]
pub enum AuthQuery {
    Account(QueryAccountRequest),
    Accounts(QueryAccountsRequest),
    Params(QueryParamsRequest),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(kind = "response")]
#[serde(untagged)]
pub enum AuthQueryResponse {
    Account(QueryAccountResponse),
    Accounts(QueryAccountsResponse),
    Params(QueryParamsResponse),
}

#[derive(Debug, Clone)]
pub struct AuthQueryHandler;

impl QueryHandler for AuthQueryHandler {
    type QueryRequest = AuthQuery;

    type QueryCommands = AuthQueryCli;

    type QueryResponse = AuthQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            AuthCommands::Account(AccountCommand { address }) => {
                AuthQuery::Account(QueryAccountRequest {
                    address: address.clone(),
                })
            }
            AuthCommands::Accounts(cmd) => {
                let pagination = PaginationRequest::try_from(cmd.to_owned().pagination)?;
                AuthQuery::Accounts(QueryAccountsRequest { pagination })
            }
            AuthCommands::Params => AuthQuery::Params(QueryParamsRequest {}),
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match command.command {
            AuthCommands::Account(_) => {
                AuthQueryResponse::Account(QueryAccountResponse::decode::<Bytes>(
                    query_bytes.into(),
                )?)
            }
            AuthCommands::Accounts(_) => AuthQueryResponse::Accounts(
                QueryAccountsResponse::decode::<Bytes>(query_bytes.into())?,
            ),
            AuthCommands::Params => {
                AuthQueryResponse::Params(QueryParamsResponse::decode::<Bytes>(query_bytes.into())?)
            }
        };

        Ok(res)
    }
}

#[derive(Args, Debug)]
pub struct TxQueryCli {
    #[command(subcommand)]
    pub command: TxCommands,
}

#[derive(Debug, Clone)]
pub enum TxQueryType {
    Hash,
    AccSeq,
    Signature,
}

impl FromStr for TxQueryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hash" => Ok(TxQueryType::Hash),
            "acc_seq" => Ok(TxQueryType::AccSeq),
            "signature" => Ok(TxQueryType::Signature),
            _ => Err("Unknown transaction type".to_string()),
        }
    }
}

impl Display for TxQueryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxQueryType::Hash => write!(f, "hash"),
            TxQueryType::AccSeq => write!(f, "acc_seq"),
            TxQueryType::Signature => write!(f, "signature"),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum TxCommands {
    Tx {
        hash: String,
        #[arg(long, default_value_t = TxQueryType::Hash)]
        query_type: TxQueryType,
    },
}

#[derive(Clone, PartialEq, Query)]
#[query(kind = "request")]
pub enum TxQuery {
    Tx(QueryGetTxRequest),
    Txs(QueryGetTxsEventRequest),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TxQueryResponse<M: TxMessage> {
    Tx(QueryTxResponse<M>),
    Txs(QueryTxsResponse<M>),
}

impl<M: TxMessage> QueryResponse for TxQueryResponse<M> {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            TxQueryResponse::Tx(msg) => msg.encode_vec(),
            TxQueryResponse::Txs(msg) => msg.encode_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TxQueryHandler<M: TxMessage> {
    msgs_type: PhantomData<M>,
}

impl<M: TxMessage> TxQueryHandler<M> {
    /// Constructor allows to handle tx messages received by query calls.
    pub fn new(_msgs_type: M) -> TxQueryHandler<M> {
        TxQueryHandler {
            msgs_type: PhantomData,
        }
    }

    fn make_tx_response(
        &self,
        tx_response: CosmosTxResponse,
        block_response: BlockResponse,
    ) -> Result<TxResponse<M>, CoreError> {
        TxResponse::new_from_tx_response_and_string_time(
            tx_response,
            block_response.block.header.time.to_string(),
        )
    }
}

impl<M: TxMessage> QueryHandler for TxQueryHandler<M> {
    type QueryRequest = TxQuery;

    type QueryCommands = TxQueryCli;

    type QueryResponse = TxQueryResponse<M>;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        match &command.command {
            TxCommands::Tx { hash, query_type } => {
                let res = match query_type {
                    TxQueryType::Hash => Self::QueryRequest::Tx(QueryGetTxRequest {
                        hash: Hash::from_str(hash)?,
                    }),
                    TxQueryType::Signature => {
                        let events = if hash.is_empty() {
                            return Err(anyhow!(
                                "Signatures list is empty. Please, provide at least one signature."
                            ));
                        } else {
                            hash.split(',')
                                .map(|sig| format!("tx.signature={sig}"))
                                .collect()
                        };
                        Self::QueryRequest::Txs(QueryGetTxsEventRequest {
                            events,
                            order_by: "asc".to_string(),
                            // default page
                            // TODO: may be part of gears constants
                            page: 1,
                            // default limit
                            // TODO: may be part of gears constants
                            limit: 100,
                        })
                    }
                    TxQueryType::AccSeq => {
                        let events = if hash.is_empty() {
                            return Err(anyhow!(
                                "Account sequence is not set. Please, provide correct value."
                            ));
                        } else {
                            vec![format!("tx.acc_seq={hash}")]
                        };
                        Self::QueryRequest::Txs(QueryGetTxsEventRequest {
                            events,
                            order_by: "asc".to_string(),
                            page: 1,
                            limit: 100,
                        })
                    }
                };
                Ok(res)
            }
        }
    }

    fn execute_query_request(
        &self,
        query: Self::QueryRequest,
        node: url::Url,
        _height: Option<gears::tendermint::types::proto::block::Height>,
    ) -> anyhow::Result<Vec<u8>> {
        let client = HttpClient::new(node.as_str())?;

        let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

        match query {
            Self::QueryRequest::Tx(req) => {
                let res: anyhow::Result<(CosmosTxResponse, BlockResponse)> =
                    runtime.block_on(async {
                        let tx_res = client.tx(req.hash, true).await?;
                        let block_res = client.block(tx_res.height).await?;
                        Ok((tx_res, block_res))
                    });
                let (tx_res, block_res) = res?;

                let res = self.make_tx_response(tx_res, block_res)?;
                Ok(res.encode_vec())
            }
            Self::QueryRequest::Txs(req) => {
                let query_str_events = req.events.join(" AND ");
                let prefix = "tm.event='Tx'";
                let mut query = String::with_capacity(prefix.len() + query_str_events.len());
                query.push_str(prefix);
                query.push_str(&query_str_events);
                let query = gears::tendermint::rpc::query::Query::from_str(&query)?;
                let res = runtime.block_on(
                    client.tx_search(
                        query,
                        true,
                        req.page,
                        req.limit.try_into().unwrap_or(u8::MAX),
                        gears::tendermint::rpc::Order::from_str(&req.order_by)?,
                    ), // TODO: extend by the other logic
                )?;

                Ok(serde_json::to_vec(&res)?)
            }
        }
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        match &command.command {
            TxCommands::Tx {
                query_type,
                hash: _,
            } => match query_type {
                TxQueryType::Hash => {
                    let tx: TxResponse<M> = TxResponse::decode_vec(&query_bytes)?;
                    Ok(Self::QueryResponse::Tx(QueryTxResponse { tx }))
                }
                _ => {
                    todo!()
                    // let res: gears::tendermint::rpc::response::tx::search::Response =
                    //     serde_json::from_slice(&query_bytes)?;
                    // Ok(Self::QueryResponse::Txs(QueryTxsResponse {
                    //     txs: res.txs.into_iter().map(Into::into).collect(),
                    // }))
                }
            },
        }
    }
}
