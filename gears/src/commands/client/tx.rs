use std::path::PathBuf;

use core_types::tx::mode_info::SignMode;
use prost::Message;
use tendermint::rpc::client::{Client, HttpClient};
use tendermint::rpc::response::tx::broadcast::Response;
use tendermint::types::chain_id::ChainId;

use crate::application::handlers::client::{TxExecutionResult, TxHandler};
use crate::commands::client::query::execute_query;
use crate::crypto::keys::ReadAccAddress;
use crate::crypto::ledger::LedgerProxyKey;
use crate::runtime::runtime;
use crate::types::base::coins::UnsignedCoins;
use crate::types::tx::raw::TxRaw;

use super::keys::KeyringBackend;

#[derive(Debug, Clone, former::Former)]
pub struct TxCommand<C> {
    pub node: url::Url,
    pub chain_id: ChainId,
    pub fees: Option<UnsignedCoins>,
    pub keyring: Keyring,
    pub inner: C,
}

#[allow(dead_code)]
pub struct ClientTxContext {
    node: url::Url,
    chain_id: ChainId,
    fees: Option<UnsignedCoins>,
    keyring: Keyring,
}

impl ClientTxContext {
    pub fn query<Response: TryFrom<Raw>, Raw: Message + Default + std::convert::From<Response>>(
        &self,
        path: String,
        query_bytes: Vec<u8>,
    ) -> anyhow::Result<Response>
    where
        <Response as TryFrom<Raw>>::Error: std::fmt::Display,
    {
        execute_query(path, query_bytes, self.node.as_str(), None)
    }
}

impl<C> From<&TxCommand<C>> for ClientTxContext {
    fn from(
        // to keep structure after changes in TxCommand
        TxCommand {
            node,
            chain_id,
            fees,
            keyring,
            inner: _,
        }: &TxCommand<C>,
    ) -> Self {
        Self {
            node: node.clone(),
            chain_id: chain_id.clone(),
            fees: fees.clone(),
            keyring: keyring.clone(),
        }
    }
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

#[derive(Debug, Clone)]
pub enum RuntxResult {
    Broadcast(Vec<Response>),
    File(PathBuf),
    None,
}

impl From<TxExecutionResult> for RuntxResult {
    fn from(value: TxExecutionResult) -> Self {
        match value {
            TxExecutionResult::Broadcast(var) => Self::Broadcast(vec![var]),
            TxExecutionResult::File(var) => Self::File(var),
            TxExecutionResult::None => Self::None,
        }
    }
}

pub fn run_tx<C, H: TxHandler<TxCommands = C>>(
    command: TxCommand<C>,
    handler: &H,
) -> anyhow::Result<RuntxResult> {
    match command.keyring {
        Keyring::Ledger => {
            let key = LedgerProxyKey::new()?;

            let ctx = &(&command).into();
            let messages = handler.prepare_tx(Some(ctx), command.inner, key.get_address())?;
            handler
                .handle_tx(
                    handler.sign_msg(
                        messages,
                        &key,
                        &command.node,
                        command.chain_id,
                        command.fees,
                        SignMode::Textual,
                    )?,
                    command.node,
                )
                .map(Into::into)
        }
        Keyring::Local(ref info) => {
            let keyring_home = info.home.join(info.keyring_backend.get_sub_dir());
            let key = keyring::key_by_name(
                &info.from_key,
                info.keyring_backend.to_keyring_backend(&keyring_home),
            )?;

            let ctx = &(&command).into();
            let messages = handler.prepare_tx(Some(ctx), command.inner, key.get_address())?;

            if messages.chunk_size() > 0
            // TODO: uncomment and update logic when command will be extended by broadcast_mode
            /* && command.broadcast_mode == BroadcastMode::Block */
            {
                let chunk_size = messages.chunk_size();
                let msgs = messages.into_msgs();

                let mut res = vec![];
                for slice in msgs.chunks(chunk_size) {
                    res.push(
                        handler
                            .handle_tx(
                                handler.sign_msg(
                                    slice
                                        .to_vec()
                                        .try_into()
                                        .expect("chunking of the messages excludes empty vectors"),
                                    &key,
                                    &command.node,
                                    command.chain_id.clone(),
                                    command.fees.clone(),
                                    SignMode::Direct,
                                )?,
                                command.node.clone(),
                            )?
                            .broadcast()
                            .ok_or(anyhow::anyhow!("tx is not broadcasted"))?,
                    );
                }
                Ok(RuntxResult::Broadcast(res))
            } else {
                // TODO: can be reduced by changing variable `step`. Do we need it?
                handler
                    .handle_tx(
                        handler.sign_msg(
                            messages,
                            &key,
                            &command.node,
                            command.chain_id,
                            command.fees,
                            SignMode::Direct,
                        )?,
                        command.node,
                    )
                    .map(Into::into)
            }
        }
    }
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> anyhow::Result<Response> {
    let res = runtime().block_on(
        client.broadcast_tx_commit(core_types::tx::raw::TxRaw::from(raw_tx).encode_to_vec()),
    )?;

    Ok(res)
}
