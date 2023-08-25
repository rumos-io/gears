use std::{path::PathBuf, str::FromStr};

use anyhow::{Ok, Result};
use auth::cli::query::get_account;
use clap::{arg, Args, Subcommand};

use gears::client::keys::key_store::DiskStore;
use ibc_proto::{
    cosmos::tx::v1beta1::{
        mode_info::{Single, Sum},
        ModeInfo, SignDoc, TxBody, TxRaw,
    },
    protobuf::Protobuf,
};
use ibc_relayer::keyring::{Secp256k1KeyPair, SigningKeyPair};
use prost::Message;
use proto_messages::cosmos::{
    bank::v1beta1::MsgSend,
    base::v1beta1::{Coin, SendCoins},
    crypto::secp256k1::v1beta1::{PubKey, RawPubKey},
    tx::v1beta1::{AuthInfo, Fee, PublicKey, SignerInfo},
};
use proto_types::AccAddress;
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

// TODO:
// 1. remove hard coded gas limit
// 2. remove hard coded chain_id

#[derive(Args, Debug)]
pub struct Cli {
    #[command(subcommand)]
    command: BankCommands,
}

#[derive(Subcommand, Debug)]
pub enum BankCommands {
    /// Send funds from one account to another
    Send {
        /// from
        from: String,
        /// to address
        to_address: AccAddress,
        /// amount
        amount: Coin,
        /// Fee to pay along with transaction
        #[arg(short, long)]
        fee: Option<Coin>,
    },
}

pub fn run_bank_tx_command(args: Cli, node: &str, home: PathBuf) -> Result<()> {
    match args.command {
        BankCommands::Send {
            from,
            to_address,
            amount,
            fee,
        } => {
            let key_store: DiskStore<Secp256k1KeyPair> = DiskStore::new(home)?;

            let key = key_store.get_key(&from)?;

            let client = HttpClient::new(node)?;
            let account = Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_account(client, AccAddress::from_str(&key.account())?))?;

            let tx_raw = create_signed_send_tx(
                AccAddress::from_str(&key.account())?,
                to_address.clone(),
                amount,
                fee,
                account.account.get_sequence(),
                account.account.get_account_number(),
                key,
            )?;

            let client = HttpClient::new(node)?;
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(broadcast_tx_commit(client, tx_raw))
        }
    }
}

pub async fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<()> {
    let res = client
        .broadcast_tx_commit(raw_tx.encode_to_vec())
        .await
        .unwrap(); //TODO: remove unwrap

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}

pub fn create_signed_send_tx(
    from_address: AccAddress,
    to_address: AccAddress,
    amount: Coin,
    fee_amount: Option<Coin>,
    sequence: u64,
    account_number: u64,
    key: Secp256k1KeyPair,
) -> Result<TxRaw> {
    let message = MsgSend {
        from_address,
        to_address,
        amount: SendCoins::new(vec![amount])?,
    };

    let tx_body = TxBody {
        messages: vec![message.into()],
        memo: "".into(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    let public_key = key.public_key.serialize().to_vec();
    let public_key = RawPubKey { key: public_key };
    let public_key: PubKey = public_key
        .try_into()
        .expect("converting the secp256k1 library's public key will always succeed");

    let signer_infos = SignerInfo {
        public_key: Some(PublicKey::Secp256k1(public_key)),
        mode_info: Some(ModeInfo {
            sum: Some(Sum::Single(Single { mode: 1 })),
        }),
        sequence,
    };

    let fee_amount = fee_amount.map(|f| SendCoins::new(vec![f])).transpose()?; // can legitimately fail if coin amount is zero

    let fee = Fee {
        amount: fee_amount,
        gas_limit: 100000000,
        payer: None,
        granter: "".into(),
    };

    let auth_info = AuthInfo {
        signer_infos: vec![signer_infos],
        fee,
        tip: None,
    };

    let sign_doc = SignDoc {
        body_bytes: tx_body.encode_to_vec(),
        auth_info_bytes: auth_info.encode_vec(),
        chain_id: "test-chain".into(), //TODO: this should be passed in
        account_number,
    };

    let signature = key.sign(&sign_doc.encode_to_vec()).unwrap(); //TODO: remove unwrap

    Ok(TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: vec![signature],
    })
}
