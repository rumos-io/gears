use std::path::PathBuf;

use anyhow::Result;
use core_types::tx::mode_info::SignMode;
use prost::Message;
use tendermint::rpc::client::{Client, HttpClient};
use tendermint::rpc::response::tx::broadcast::Response;
use tendermint::types::chain_id::ChainId;

use crate::application::handlers::client::TxHandler;
use crate::crypto::keys::ReadAccAddress;
use crate::crypto::ledger::LedgerProxyKey;
use crate::runtime::runtime;
use crate::types::base::send::SendCoins;
use crate::types::tx::raw::TxRaw;

use super::keys::KeyringBackend;

#[derive(Debug, Clone, former::Former)]
pub struct TxCommand<C> {
    pub node: url::Url,
    pub chain_id: ChainId,
    pub fee: Option<SendCoins>,
    pub keyring: Keyring,
    pub inner: C,
}

#[derive(Debug, Clone)]
pub enum Keyring {
    Ledger,
    Local(LocalInfo),
}

#[derive(Debug, Clone)]
pub struct LocalInfo {
    pub keyring_backend: KeyringBackend,
    pub from_key: String,
    pub home: PathBuf,
}

pub fn run_tx<C, H: TxHandler<TxCommands = C>>(
    TxCommand {
        node,
        chain_id,
        fee,
        inner,
        keyring,
    }: TxCommand<C>,
    handler: &H,
) -> anyhow::Result<Response> {
    match keyring {
        Keyring::Ledger => {
            let key = LedgerProxyKey::new()?;

            let message = handler.prepare_tx(inner, key.get_address())?;
            handler.handle_tx(message, key, node, chain_id, fee, SignMode::Textual)
        }
        Keyring::Local(info) => {
            let keyring_home = info.home.join(info.keyring_backend.get_sub_dir());
            let key = keyring::key_by_name(
                &info.from_key,
                info.keyring_backend.to_keyring_backend(&keyring_home),
            )?;

            let message = handler.prepare_tx(inner, key.get_address())?;
            handler.handle_tx(message, key, node, chain_id, fee, SignMode::Direct)
        }
    }
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<Response> {
    let res = runtime().block_on(
        client.broadcast_tx_commit(core_types::tx::raw::TxRaw::from(raw_tx).encode_to_vec()),
    )?;

    Ok(res)
}
