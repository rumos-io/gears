use std::path::PathBuf;

use anyhow::Result;
use prost::Message;
use proto_messages::cosmos::{
    auth::v1beta1::{QueryAccountRequest, QueryAccountResponse},
    base::v1beta1::SendCoins,
    ibc::{auth::RawQueryAccountResponse, protobuf::Protobuf, tx::TxRaw},
    tx::v1beta1::{fee::Fee, message::Message as SDKMessage, tx_body::TxBody},
};
use proto_types::AccAddress;
use tendermint::informal::chain::Id;
use tendermint::rpc::{Client, HttpClient};

use crate::{client::keys::KeyringBackend, TxHandler};
use crate::crypto::{create_signed_transaction, SigningInfo};

use super::query::run_query;

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TxCommand<C>
{
    pub home: PathBuf,
    pub node : tendermint::rpc::Url,
    pub from_key : String,
    pub chain_id: Id,
    pub fee : Option<SendCoins>,
    pub keyring_backend: KeyringBackend,

    pub inner : C,
}

pub async fn run_tx_command<M : SDKMessage, C, H : TxHandler< TxCommands = C >>(
    cmd : TxCommand<C>,
    handler : &H,
) -> Result<()>
{
    let TxCommand { home, node, from_key, chain_id, fee, keyring_backend, inner : _ } = &cmd;

    let keyring_home = home.join(keyring_backend.get_sub_dir());

    let key = keyring::get_key_by_name(from_key, keyring_backend.to_keyring_backend(&keyring_home))?;
    let address = key.get_address();

    let message = handler.handle_tx_command(&cmd, address.clone())?;

    let fee = Fee {
        amount: fee.clone(),
        gas_limit: 100000000, //TODO: remove hard coded gas limit
        payer: None,          //TODO: remove hard coded payer
        granter: "".into(),   //TODO: remove hard coded granter
    };

    let account = get_account_latest(address, node.path())?;

    let signing_info = SigningInfo {
        key,
        sequence: account.account.get_sequence(),
        account_number: account.account.get_account_number(),
    };

    let tx_body = TxBody {
        messages: vec![message],
        memo: String::new(),                     // TODO: remove hard coded
        timeout_height: 0,                      // TODO: remove hard coded
        extension_options: vec![],              // TODO: remove hard coded
        non_critical_extension_options: vec![], // TODO: remove hard coded
    };

    let tip = None; //TODO: remove hard coded

    let raw_tx = create_signed_transaction(vec![signing_info], tx_body, fee, tip, chain_id.clone());

    let client = HttpClient::new(node.clone())?;

    broadcast_tx_commit(client, raw_tx).await
}

pub async fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<()> {
    let res = client.broadcast_tx_commit(raw_tx.encode_to_vec()).await?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}

// TODO: we're assuming here that the app has an auth module which handles this query
fn get_account_latest(address: AccAddress, node: &str) -> Result<QueryAccountResponse> {
    let query = QueryAccountRequest { address };

    run_query::<QueryAccountResponse, RawQueryAccountResponse>(
        query.encode_vec(),
        "/cosmos.auth.v1beta1.Query/Account".into(),
        node,
        None,
    )
}
