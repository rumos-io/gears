use std::path::PathBuf;

use anyhow::Result;
use keyring::key_by_name;
use prost::Message;
use tendermint::rpc::client::{Client, HttpClient};
use tendermint::rpc::response::tx::broadcast::Response;
use tendermint::types::chain_id::ChainId;

use crate::application::handlers::client::TxHandler;
use crate::crypto::keys::ReadAccAddress;
use crate::runtime::runtime;
use crate::types::base::send::SendCoins;
use crate::types::tx::raw::TxRaw;

use super::keys::KeyringBackend;

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TxCommand<C> {
    pub home: PathBuf,
    pub node: url::Url,
    pub from_key: String,
    pub chain_id: ChainId,
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
) -> anyhow::Result<Response> {
    let keyring_home = home.join(keyring_backend.get_sub_dir());

    let key = key_by_name(&from_key, keyring_backend.to_keyring_backend(&keyring_home))?;

    let message = handler.prepare_tx(inner, key.get_address())?;

    handler.handle_tx(message, key, node, chain_id, fee)
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<Response> {
    let res = runtime().block_on(
        client.broadcast_tx_commit(ibc_types::tx::raw::TxRaw::from(raw_tx).encode_to_vec()),
    )?;

    Ok(res)
}
