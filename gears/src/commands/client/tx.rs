use std::path::PathBuf;

use core_types::tx::mode_info::SignMode;
use prost::Message;
use tendermint::rpc::client::{Client, HttpClient};
use tendermint::rpc::response::tx::broadcast::Response;
use tendermint::types::chain_id::ChainId;

use crate::application::handlers::client::{TxExecutionResult, TxHandler};
use crate::commands::client::query::execute_query;
use crate::crypto::any_key::AnyKey;
use crate::crypto::keys::GearsPublicKey;
use crate::crypto::ledger::LedgerProxyKey;
use crate::runtime::runtime;
use crate::types::base::coins::UnsignedCoins;
use crate::types::tx::raw::TxRaw;

use super::keys::KeyringBackend;

#[derive(Debug, Clone)]
pub enum AccountProvider {
    Offline { sequence: u64, account_number: u64 },
    Online,
}

#[derive(Debug, Clone, former::Former)]
pub struct TxCommand<C> {
    pub home: PathBuf,
    pub node: url::Url,
    pub chain_id: ChainId,
    pub account: AccountProvider,
    pub fees: Option<UnsignedCoins>,
    pub keyring: Keyring,
    pub inner: C,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClientTxContext {
    pub node: url::Url,
    pub home: PathBuf,
    pub keyring: Keyring,
    pub memo: Option<String>,
    pub account: AccountProvider,
    chain_id: ChainId,
    fees: Option<UnsignedCoins>,
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
            home,
            node,
            chain_id,
            fees,
            keyring,
            account,
            inner: _,
        }: &TxCommand<C>,
    ) -> Self {
        Self {
            home: home.clone(),
            node: node.clone(),
            chain_id: chain_id.clone(),
            fees: fees.clone(),
            keyring: keyring.clone(),
            account: account.clone(),
            memo: None,
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
}

#[derive(Debug, Clone)]
pub enum RuntxResult {
    Broadcast(Vec<Response>),
    File(PathBuf),
    None,
}

impl RuntxResult {
    pub fn broadcast(self) -> Option<Vec<Response>> {
        match self {
            Self::Broadcast(var) => Some(var),
            Self::File(_) => None,
            Self::None => None,
        }
    }

    pub fn file(self) -> Option<PathBuf> {
        match self {
            Self::Broadcast(_) => None,
            Self::File(var) => Some(var),
            Self::None => None,
        }
    }
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

fn handle_key(client_tx_context: &ClientTxContext) -> anyhow::Result<AnyKey> {
    match client_tx_context.keyring {
        Keyring::Ledger => Ok(AnyKey::Ledger(LedgerProxyKey::new()?)),
        Keyring::Local(ref local) => {
            let keyring_home = client_tx_context
                .home
                .join(local.keyring_backend.get_sub_dir());
            let key = keyring::key_by_name(
                &local.from_key,
                local.keyring_backend.to_keyring_backend(&keyring_home),
            )?;

            Ok(AnyKey::Local(key))
        }
    }
}

pub fn run_tx<C, H: TxHandler<TxCommands = C>>(
    command: TxCommand<C>,
    handler: &H,
) -> anyhow::Result<RuntxResult> {
    let ctx = &mut (&command).into();
    let key = handle_key(ctx)?;

    let messages = handler.prepare_tx(ctx, command.inner, key.get_gears_public_key())?;

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
                            ctx,
                        )?,
                        ctx,
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
                    ctx,
                )?,
                ctx,
            )
            .map(Into::into)
    }
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> anyhow::Result<Response> {
    let res = runtime().block_on(
        client.broadcast_tx_commit(core_types::tx::raw::TxRaw::from(raw_tx).encode_to_vec()),
    )?;

    Ok(res)
}
