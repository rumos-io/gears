use keyring::pair::KeyPair;
// use proto_messages::cosmos::{query::Query, tx::v1beta1::message::Message};
use proto_types::{coin::send::SendCoins, tx::TxMessage, AccAddress};
use serde::Serialize;
use tendermint::{
    rpc::{client::HttpClient, response::tx::Response},
    types::{chain_id::ChainId, proto::block::Height},
};

use crate::runtime::runtime;
// use tendermint::{
//     informal::block::Height,
//     rpc::{endpoint::broadcast::tx_commit::Response, Client, HttpClient},
// };

// use crate::{
//     client::{query::execute_query, tx::broadcast_tx_commit},
//     crypto::{create_signed_transaction, SigningInfo},
//     runtime::runtime,
// };
// use proto_messages::cosmos::{
//     auth::v1beta1::{QueryAccountRequest, QueryAccountResponse},
//     base::v1beta1::SendCoins,
//     ibc::{auth::RawQueryAccountResponse, protobuf::Protobuf},
//     tx::v1beta1::{fee::Fee, tx_body::TxBody},
// };

pub trait TxHandler {
    type Message: TxMessage;
    type TxCommands;

    fn prepare_tx(
        &self,
        command: Self::TxCommands,
        from_address: AccAddress,
    ) -> anyhow::Result<Self::Message>;

    fn handle_tx(
        &self,
        msg: Self::Message,
        key: KeyPair,
        node: url::Url,
        chain_id: ChainId,
        fee: Option<SendCoins>,
    ) -> anyhow::Result<Response> {
        let fee = Fee {
            amount: fee,
            gas_limit: 100000000, //TODO: remove hard coded gas limit
            payer: None,          //TODO: remove hard coded payer
            granter: "".into(),   //TODO: remove hard coded granter
        };

        let address = key.get_address();

        let account = get_account_latest(address, node.as_str())?;

        let signing_info = SigningInfo {
            key,
            sequence: account.account.get_sequence(),
            account_number: account.account.get_account_number(),
        };

        let tx_body = TxBody {
            messages: vec![msg],
            memo: String::new(),                    // TODO: remove hard coded
            timeout_height: 0,                      // TODO: remove hard coded
            extension_options: vec![],              // TODO: remove hard coded
            non_critical_extension_options: vec![], // TODO: remove hard coded
        };

        let tip = None; //TODO: remove hard coded

        let raw_tx = create_signed_transaction(vec![signing_info], tx_body, fee, tip, chain_id);

        let client = HttpClient::new(tendermint::rpc::Url::try_from(node)?)?;

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
            Some(query.query_url().into_owned()),
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

/// Name aux stands for `auxiliary`. In terms of implementation this is more like user extension to CLI.
/// It's reason exists to add user specific commands which doesn't supports usually.
#[allow(unused_variables)]
pub trait AuxHandler {
    type AuxCommands; // TODO: use NilAuxCommand as default if/when associated type defaults land https://github.com/rust-lang/rust/issues/29661
    type Aux;

    fn prepare_aux(&self, command: Self::AuxCommands) -> anyhow::Result<Self::Aux> {
        Err(anyhow::anyhow!("unimplemented"))
    }

    fn handle_aux(&self, aux: Self::Aux) -> anyhow::Result<()> {
        Ok(())
    }
}

// TODO: we're assuming here that the app has an auth module which handles this query
fn get_account_latest(address: AccAddress, node: &str) -> anyhow::Result<QueryAccountResponse> {
    let query = QueryAccountRequest { address };

    execute_query::<QueryAccountResponse, RawQueryAccountResponse>(
        "/cosmos.auth.v1beta1.Query/Account".into(),
        query.encode_vec(),
        node,
        None,
    )
}
