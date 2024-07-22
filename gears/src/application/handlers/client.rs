use crate::{
    baseapp::Query,
    commands::client::{
        query::execute_query,
        tx::{broadcast_tx_commit, ClientTxContext},
    },
    crypto::{
        info::{create_signed_transaction_direct, create_signed_transaction_textual, SigningInfo},
        keys::{GearsPublicKey, ReadAccAddress, SigningKey},
    },
    error::IBC_ENCODE_UNWRAP,
    runtime::runtime,
    signing::{handler::MetadataGetter, renderer::value_renderer::ValueRenderer},
    types::{
        address::AccAddress,
        auth::fee::Fee,
        base::coins::UnsignedCoins,
        denom::Denom,
        query::{
            // account::{QueryAccountRequest, QueryAccountResponse},
            metadata::{
                QueryDenomMetadataRequest, QueryDenomMetadataResponse,
                RawQueryDenomMetadataResponse,
            },
        },
        tx::{body::TxBody, Messages, TxMessage},
    },
};

use models::{QueryAccountRequest, QueryAccountResponse};
use tendermint::types::proto::Protobuf as TMProtobuf;

use anyhow::anyhow;
use core_types::tx::mode_info::SignMode;
use serde::Serialize;

use tendermint::{
    rpc::{
        client::{Client, HttpClient},
        response::tx::broadcast::Response,
    },
    types::{chain_id::ChainId, proto::block::Height},
};

pub trait TxHandler {
    type Message: TxMessage + ValueRenderer;
    type TxCommands;

    fn prepare_tx(
        &self,
        client_tx_context: &ClientTxContext,
        command: Self::TxCommands,
        from_address: AccAddress,
    ) -> anyhow::Result<Messages<Self::Message>>;

    fn handle_tx<K: SigningKey + ReadAccAddress + GearsPublicKey>(
        &self,
        msgs: Messages<Self::Message>,
        key: &K,
        node: url::Url,
        chain_id: ChainId,
        fees: Option<UnsignedCoins>,
        mode: SignMode,
    ) -> anyhow::Result<Response> {
        let fee = Fee {
            amount: fees,
            gas_limit: 200_000_u64
                .try_into()
                .expect("hard coded gas limit is valid"), //TODO: remove hard coded gas limit
            payer: None,        //TODO: remove hard coded payer
            granter: "".into(), //TODO: remove hard coded granter
        };

        let address = key.get_address();

        let account = get_account_latest(address, node.as_str())?;

        let account = account
            .account
            .ok_or_else(|| anyhow!("account not found"))?;

        let signing_infos = vec![SigningInfo {
            key,
            sequence: account.get_sequence(),
            account_number: account.get_account_number(),
        }];

        let tx_body = TxBody {
            messages: msgs.into_msgs(),
            memo: String::new(),                    // TODO: remove hard coded
            timeout_height: 0,                      // TODO: remove hard coded
            extension_options: vec![],              // TODO: remove hard coded
            non_critical_extension_options: vec![], // TODO: remove hard coded
        };

        let tip = None; //TODO: remove hard coded

        let raw_tx = match mode {
            SignMode::Direct => create_signed_transaction_direct(
                signing_infos,
                chain_id,
                fee,
                tip,
                tx_body.encode_vec().expect(IBC_ENCODE_UNWRAP),
            )
            .map_err(|e| anyhow!(e.to_string()))?,
            SignMode::Textual => create_signed_transaction_textual(
                signing_infos,
                chain_id,
                fee,
                tip,
                node.clone(),
                tx_body,
            )
            .map_err(|e| anyhow!(e.to_string()))?,
            _ => return Err(anyhow!("unsupported sign mode")),
        };

        let client = HttpClient::new(tendermint::rpc::url::Url::try_from(node)?)?;
        broadcast_tx_commit(client, raw_tx)
    }
}

/// Handles query request, serialization and displaying it as `String`
pub trait QueryHandler {
    /// Query request which contains all information needed for request
    type QueryRequest: Query;
    /// Additional context to use. \
    /// In most cases you would expect this to be some sort of cli command
    type QueryCommands;
    /// Serialized response from query request
    type QueryResponse: Serialize;

    /// Prepare query to execute based on input command.
    /// Return `Self::Query` which should be used in `Self::execute_query` to retrieve raw bytes of query
    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest>;

    /// Executes request to node
    /// Returns raw bytes of `Self::QueryResponse`
    fn execute_query_request(
        &self,
        query: Self::QueryRequest,
        node: url::Url,
        height: Option<Height>,
    ) -> anyhow::Result<Vec<u8>> {
        let client = HttpClient::new(node.as_str())?;

        let res = runtime().block_on(client.abci_query(
            Some(query.query_url().to_owned()),
            query.into_bytes(),
            height,
            false,
        ))?;

        if res.code.is_err() {
            return Err(anyhow::anyhow!("node returned an error: {}", res.log));
        }

        Ok(res.value)
    }

    /// Handle serialization of query bytes into concrete type. \
    /// # Motivation
    /// This method allows to use custom serialization logic without introducing any new trait bound
    /// and allows to use it with enum which stores all responses from you module
    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse>;
}

mod inner {
    pub use core_types::query::response::account::QueryAccountResponse;
}

// TODO: we're assuming here that the app has an auth module which handles this query
pub(crate) fn get_account_latest(
    address: AccAddress,
    node: &str,
) -> anyhow::Result<QueryAccountResponse> {
    let query = QueryAccountRequest { address };

    execute_query::<QueryAccountResponse, inner::QueryAccountResponse>(
        "/cosmos.auth.v1beta1.Query/Account".into(),
        query.encode_vec()?,
        node,
        None,
    )
}

// TODO: we're assuming here that the app has a bank module which handles this query
pub(crate) fn get_denom_metadata(
    base: Denom,
    node: &str,
) -> anyhow::Result<QueryDenomMetadataResponse> {
    let query = QueryDenomMetadataRequest { denom: base };

    execute_query::<QueryDenomMetadataResponse, RawQueryDenomMetadataResponse>(
        "/cosmos.bank.v1beta1.Query/DenomMetadata".into(),
        query.encode_vec().expect(IBC_ENCODE_UNWRAP), // TODO:IBC
        node,
        None,
    )
}

pub struct MetadataViaRPC {
    pub node: url::Url,
}

impl MetadataGetter for MetadataViaRPC {
    type Error = anyhow::Error;

    fn metadata(
        &self,
        denom: &Denom,
    ) -> Result<Option<crate::types::tx::metadata::Metadata>, Self::Error> {
        let res = get_denom_metadata(denom.to_owned(), self.node.as_str())?;
        Ok(res.metadata)
    }
}

// TODO: Rework `get_account_latest`
mod models {
    use serde::{Deserialize, Serialize};
    use tendermint::types::proto::Protobuf;

    use crate::types::{account::Account, address::AccAddress};

    mod inner {
        pub use core_types::query::request::account::QueryAccountRequest;
        pub use core_types::query::response::account::QueryAccountResponse;
    }

    /// QueryAccountResponse is the response type for the Query/Account RPC method.
    #[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
    pub struct QueryAccountResponse {
        /// account defines the account of the corresponding address.
        pub account: Option<Account>,
    }

    impl TryFrom<inner::QueryAccountResponse> for QueryAccountResponse {
        type Error = core_types::errors::CoreError;

        fn try_from(raw: inner::QueryAccountResponse) -> Result<Self, Self::Error> {
            let account = raw.account.map(|a| a.try_into()).transpose()?;
            Ok(QueryAccountResponse { account })
        }
    }

    impl From<QueryAccountResponse> for inner::QueryAccountResponse {
        fn from(query: QueryAccountResponse) -> inner::QueryAccountResponse {
            Self {
                account: query.account.map(Into::into),
            }
        }
    }

    impl Protobuf<inner::QueryAccountResponse> for QueryAccountResponse {}

    /// QueryAccountRequest is the request type for the Query/Account RPC method.
    #[derive(Clone, PartialEq, Debug)]
    pub struct QueryAccountRequest {
        /// address defines the address to query for.
        pub address: AccAddress,
    }

    impl TryFrom<inner::QueryAccountRequest> for QueryAccountRequest {
        type Error = core_types::errors::CoreError;

        fn try_from(raw: inner::QueryAccountRequest) -> Result<Self, Self::Error> {
            let address = AccAddress::from_bech32(&raw.address)
                .map_err(|e| Self::Error::DecodeAddress(e.to_string()))?;

            Ok(QueryAccountRequest { address })
        }
    }

    impl From<QueryAccountRequest> for inner::QueryAccountRequest {
        fn from(query: QueryAccountRequest) -> inner::QueryAccountRequest {
            Self {
                address: query.address.to_string(),
            }
        }
    }

    impl Protobuf<inner::QueryAccountRequest> for QueryAccountRequest {}
}
