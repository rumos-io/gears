use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use cosmwasm_std::Uint256;
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
use proto_types::{AccAddress, Denom};
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

use crate::{client::keys::key_store::DiskStore, x::auth::client::cli::query::get_account};

// TODO: remove hard coded fee

pub fn get_bank_tx_command() -> Command {
    Command::new("bank")
        .about("Bank transaction subcommands")
        .subcommand(
            Command::new("send")
                .about("Send funds from one account to another")
                .arg(
                    Arg::new("from_address")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    Arg::new("to_address")
                        .required(true)
                        .value_parser(clap::value_parser!(AccAddress)),
                )
                .arg(
                    Arg::new("amount")
                        .required(true)
                        .value_parser(clap::value_parser!(Uint256)),
                )
                .arg(
                    Arg::new("denom")
                        .required(true)
                        .value_parser(clap::value_parser!(Denom)),
                ),
        )
        .subcommand_required(true)
}

pub fn run_bank_tx_command(matches: &ArgMatches, node: &str, home: PathBuf) -> Result<()> {
    match matches.subcommand() {
        Some(("send", sub_matches)) => {
            let from = sub_matches
                .get_one::<String>("from_address")
                .expect("from address argument is required preventing `None`")
                .to_owned();

            let to_address = sub_matches
                .get_one::<AccAddress>("to_address")
                .expect("to address argument is required preventing `None`")
                .to_owned();

            let amount = sub_matches
                .get_one::<Uint256>("amount")
                .expect("amount argument is required preventing `None`")
                .to_owned();

            let denom = sub_matches
                .get_one::<Denom>("denom")
                .expect("denom argument is required preventing `None`")
                .to_owned();

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
                denom.clone(),
                account.account.get_sequence(),
                account.account.get_account_number(),
                key,
            )?;

            let client = HttpClient::new(node)?;
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(broadcast_tx_commit(client, tx_raw))
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub async fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<()> {
    let res = client
        .broadcast_tx_commit(raw_tx.encode_to_vec())
        .await
        .unwrap();

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}

pub fn create_signed_send_tx(
    from_address: AccAddress,
    to_address: AccAddress,
    amount: Uint256,
    denom: Denom,
    sequence: u64,
    account_number: u64,
    key: Secp256k1KeyPair,
) -> Result<TxRaw> {
    let message = MsgSend {
        from_address,
        to_address,
        amount: SendCoins::new(vec![Coin { denom, amount }])?,
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

    let fee_amount = Coin {
        denom: String::from("uatom")
            .try_into()
            .expect("uatom is a valid denom"),
        amount: Uint256::one(),
    };
    let fee_amount = SendCoins::new(vec![fee_amount])
        .expect("conversion of hard coded vector of coins will always succeed");
    let fee = Fee {
        amount: Some(fee_amount),
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
        auth_info_bytes: auth_info
            .encode_vec()
            .expect("library call will never return an error - this is a bug in the library"),
        chain_id: "localnet".into(),
        account_number,
    };

    let signature = key.sign(&sign_doc.encode_to_vec()).unwrap();

    Ok(TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: vec![signature],
    })
}
