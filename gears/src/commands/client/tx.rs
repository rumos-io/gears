use std::path::PathBuf;

use anyhow::Result;
use core_types::tx::mode_info::{ModeInfo, SignMode};
use ledger_cosmos::CosmosValidatorApp;
use prost::Message;
use tendermint::rpc::client::{Client, HttpClient};
use tendermint::rpc::response::tx::broadcast::Response;
use tendermint::types::chain_id::ChainId;

use crate::application::handlers::client::{get_account_latest, get_denom_metadata, TxHandler};
use crate::crypto::keys::ReadAccAddress;
use crate::crypto::public::PublicKey;
use crate::crypto::secp256k1::Secp256k1PubKey;
use crate::error::IBC_ENCODE_UNWRAP;
use crate::runtime::runtime;
use crate::signing::handler::SignModeHandler;
use crate::types::auth::fee::Fee;
use crate::types::auth::info::AuthInfo;
use crate::types::base::send::SendCoins;
use crate::types::denom::Denom;
use crate::types::signing::SignerInfo;
use crate::types::tx::body::TxBody;
use crate::types::tx::data::TxData;
use crate::types::tx::metadata::Metadata;
use crate::types::tx::raw::TxRaw;
use crate::types::tx::signer::SignerData;
use tendermint::types::proto::Protobuf;

use super::keys::KeyringBackend;

#[derive(Debug, Clone, derive_builder::Builder)]
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
            let cva = CosmosValidatorApp::connect().unwrap(); //TODO: unwrap
            let pub_key_raw = cva.public_key_secp256k1().unwrap(); // TODO: unwrap

            let public_key = Secp256k1PubKey::try_from(pub_key_raw.to_vec()).unwrap(); // TODO: unwrap
            let pub_key: PublicKey = public_key.into();

            let address = pub_key.get_address();

            let sign_mode_handler = SignModeHandler;

            let f = |denom: &Denom| -> Option<Metadata> {
                let res = get_denom_metadata(denom.to_owned(), node.as_str()).unwrap(); //TODO: unwrap
                res.metadata
            };

            let account = get_account_latest(address.clone(), node.as_str())?;

            let signer_data = SignerData {
                address: address.clone(),
                chain_id,
                account_number: account.account.get_account_number(),
                sequence: account.account.get_sequence(),
                pub_key: pub_key.clone(),
            };

            let message = handler.prepare_tx(inner, address.clone())?;

            // TODO: we're not using handle_tx in here

            let tx_body = TxBody {
                messages: vec![message],
                memo: String::new(),                    // TODO: remove hard coded
                timeout_height: 0,                      // TODO: remove hard coded
                extension_options: vec![],              // TODO: remove hard coded
                non_critical_extension_options: vec![], // TODO: remove hard coded
            };

            let signer_infos = vec![SignerInfo {
                public_key: Some(pub_key),
                mode_info: ModeInfo::Single(SignMode::Textual),
                sequence: account.account.get_sequence(),
            }];

            let auth_info = AuthInfo {
                signer_infos,
                fee: Fee {
                    amount: fee,
                    gas_limit: 200_000, //TODO: remove hard coded gas limit
                    payer: None,        //TODO: remove hard coded payer
                    granter: "".into(), //TODO: remove hard coded granter
                },
                tip: None, //TODO: remove hard coded
            };

            let tx_data = TxData {
                body: tx_body.clone(),
                auth_info: auth_info.clone(),
            };

            let sign_bytes = sign_mode_handler
                .sign_bytes_get(&f, signer_data, tx_data)
                .unwrap(); //todo unwrap

            println!("sign_bytes: {:?}", sign_bytes);

            let signature = cva.sign_v2(&sign_bytes).unwrap(); // TODO: unwrap

            // convert signature from DER to BER
            let signature = secp256k1::ecdsa::Signature::from_der(&signature).unwrap(); // TODO: unwrap
            let signature = signature.serialize_compact().to_vec();

            println!("signature: {:?}", signature);

            let body_bytes = tx_body.encode_vec().expect(IBC_ENCODE_UNWRAP); // TODO:IBC
            let auth_info_bytes = auth_info.encode_vec().expect(IBC_ENCODE_UNWRAP); // TODO:IBC

            let raw_tx = TxRaw {
                body_bytes,
                auth_info_bytes,
                signatures: vec![signature],
            };

            let client = HttpClient::new(tendermint::rpc::url::Url::try_from(node)?)?;

            broadcast_tx_commit(client, raw_tx)
        }
        Keyring::Local(info) => {
            let keyring_home = info.home.join(info.keyring_backend.get_sub_dir());
            let key = keyring::key_by_name(
                &info.from_key,
                info.keyring_backend.to_keyring_backend(&keyring_home),
            )?;

            // TODO: move these out of Keyring::Local match arm if necessary
            let message = handler.prepare_tx(inner, key.get_address())?;
            handler.handle_tx(message, key, node, chain_id, fee)
        }
    }
}

pub fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<Response> {
    let res = runtime().block_on(
        client.broadcast_tx_commit(core_types::tx::raw::TxRaw::from(raw_tx).encode_to_vec()),
    )?;

    Ok(res)
}
