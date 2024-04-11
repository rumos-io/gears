use std::path::PathBuf;

use anyhow::Result;
use ledger_cosmos::CosmosValidatorApp;
use prost::Message;
use proto_messages::cosmos::{base::v1beta1::SendCoins, ibc::tx::TxRaw};
use tendermint::informal::chain::Id;
use tendermint::rpc::endpoint::broadcast::tx_commit::Response;
use tendermint::rpc::{Client, HttpClient};

use crate::application::handlers::TxHandler;
use crate::client::keys::KeyringBackend;
use crate::runtime::runtime;

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct TxCommand<C> {
    pub node: url::Url,
    pub chain_id: Id,
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
    let key = match keyring {
        Keyring::Ledger => {
            let cva = CosmosValidatorApp::connect().unwrap();
            let pub_key = cva.public_key_secp256k1().unwrap();
            println!("pub_key: {:?}", pub_key);

            todo!()
        }
        Keyring::Local(info) => {
            let keyring_home = info.home.join(info.keyring_backend.get_sub_dir());
            keyring::get_key_by_name(
                &info.from_key,
                info.keyring_backend.to_keyring_backend(&keyring_home),
            )?
        }
    };

    let message = handler.prepare_tx(inner, key.get_address())?;

    handler.handle_tx(message, key, node, chain_id, fee)
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<Response> {
    let res = runtime().block_on(client.broadcast_tx_commit(raw_tx.encode_to_vec()))?;

    Ok(res)
}
