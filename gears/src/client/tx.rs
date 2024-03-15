use std::path::PathBuf;

use anyhow::Result;
use prost::Message;
use proto_messages::cosmos::{base::v1beta1::SendCoins, ibc::tx::TxRaw};
use tendermint::informal::chain::Id;
use tendermint::rpc::{Client, HttpClient};

use crate::application::handlers::TxHandler;
use crate::client::keys::KeyringBackend;
use crate::runtime::runtime;

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TxCommand<C> {
    pub home: PathBuf,
    pub node: url::Url,
    pub from_key: String,
    pub chain_id: Id,
    pub fee: Option<SendCoins>,
    pub keyring_backend: KeyringBackend,

    pub inner: C,
}

pub fn run_tx<C, H: TxHandler<TxCommands = C>>(
    TxCommand {
        home,
        node,
        from_key,
        chain_id,
        fee,
        keyring_backend,
        inner,
    }: TxCommand<C>,
    handler: &H,
) -> anyhow::Result<()> {
    let keyring_home = home.join(keyring_backend.get_sub_dir());

    let key =
        keyring::get_key_by_name(&from_key, keyring_backend.to_keyring_backend(&keyring_home))?;

    let message = handler.prepare_tx(inner, key.get_address())?;

    handler.handle_tx(message, key, node, chain_id, fee)
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<()> {
    let res = runtime().block_on(client.broadcast_tx_commit(raw_tx.encode_to_vec()))?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
