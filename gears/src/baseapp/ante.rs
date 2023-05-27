use database::DB;
use ibc_proto::cosmos::tx::v1beta1::SignDoc;
use prost::Message;
use proto_messages::cosmos::tx::v1beta1::PublicKey;
use secp256k1::{ecdsa, hashes::sha256, PublicKey as Secp256k1PubKey, Secp256k1};

use crate::{
    error::AppError,
    types::{Context, DecodedTx},
    x::{
        auth::{Auth, Module, Params as AuthParams},
        bank::Bank,
    },
};

pub struct AnteHandler {}

impl AnteHandler {
    pub fn run<T: DB>(ctx: &mut Context<T>, tx: &DecodedTx) -> Result<(), AppError> {
        validate_basic_ante_handler(tx)?;
        tx_timeout_height_ante_handler(ctx, tx)?;
        validate_memo_ante_handler(ctx, tx)?;
        deduct_fee_ante_handler(ctx, tx)?;
        set_pub_key_ante_handler(ctx, tx)?;
        sig_verification_handler(ctx, tx)?;
        increment_sequence_ante_handler(ctx, tx)?;

        //  ** ante.NewSetUpContextDecorator(),
        //  - ante.NewRejectExtensionOptionsDecorator(), // Covered in tx parsing code
        //  - NewMempoolFeeDecorator(opts.BypassMinFeeMsgTypes), // NOT USED FOR DELIVER_TX
        //  - ante.NewValidateBasicDecorator(),
        //  - ante.NewTxTimeoutHeightDecorator(),
        //  - ante.NewValidateMemoDecorator(opts.AccountKeeper),
        //  ** ante.NewConsumeGasForTxSizeDecorator(opts.AccountKeeper),
        //  - ante.NewDeductFeeDecorator(opts.AccountKeeper, opts.BankKeeper, opts.FeegrantKeeper),
        // // SetPubKeyDecorator must be called before all signature verification decorators
        //  - ante.NewSetPubKeyDecorator(opts.AccountKeeper),
        //  ** ante.NewValidateSigCountDecorator(opts.AccountKeeper),
        //  ** ante.NewSigGasConsumeDecorator(opts.AccountKeeper, sigGasConsumer),
        //  - ante.NewSigVerificationDecorator(opts.AccountKeeper, opts.SignModeHandler),
        //  - ante.NewIncrementSequenceDecorator(opts.AccountKeeper),
        //  ** ibcante.NewAnteDecorator(opts.IBCkeeper),

        Ok(())
    }
}

fn validate_basic_ante_handler(tx: &DecodedTx) -> Result<(), AppError> {
    // Not sure if we need to explicitly check this given the check which follows.
    // We'll leave it in for now since it's in the SDK.
    let sigs = tx.get_signatures();
    if sigs.is_empty() {
        return Err(AppError::TxValidation("signature list is empty".into()));
    }

    if sigs.len() != tx.get_signers().len() {
        return Err(AppError::TxValidation(format!(
            "wrong number of signatures; expected {}, got {}",
            tx.get_signers().len(),
            sigs.len()
        )));
    }

    return Ok(());
}

fn tx_timeout_height_ante_handler<T: DB>(ctx: &Context<T>, tx: &DecodedTx) -> Result<(), AppError> {
    let timeout_height = tx.get_timeout_height();

    // timeout_height of zero means no timeout height
    if timeout_height == 0 {
        return Ok(());
    }

    let block_height = ctx.get_height();

    if ctx.get_height() > timeout_height {
        return Err(AppError::Timeout {
            timeout: timeout_height,
            current: block_height,
        });
    }

    Ok(())
}

fn validate_memo_ante_handler<T: DB>(ctx: &Context<T>, tx: &DecodedTx) -> Result<(), AppError> {
    let max_memo_chars = AuthParams::get(ctx).max_memo_characters;
    let memo_length: u64 = tx
        .get_memo()
        .len()
        .try_into()
        .map_err(|_| AppError::Memo(max_memo_chars))?;

    if memo_length > max_memo_chars {
        return Err(AppError::Memo(max_memo_chars));
    };
    Ok(())
}

fn deduct_fee_ante_handler<T: DB>(ctx: &mut Context<T>, tx: &DecodedTx) -> Result<(), AppError> {
    let fee = tx.get_fee();
    let fee_payer = tx.get_fee_payer();

    if !Auth::has_account(ctx, fee_payer) {
        return Err(AppError::AccountNotFound);
    }

    if let Some(fee) = fee {
        Bank::send_coins_from_account_to_module(
            ctx,
            fee_payer.to_owned(),
            Module::FeeCollector,
            fee.to_owned(),
        )?;
    }

    Ok(())
}

fn set_pub_key_ante_handler<T: DB>(ctx: &mut Context<T>, tx: &DecodedTx) -> Result<(), AppError> {
    let public_keys = tx.get_public_keys();
    let signers = tx.get_signers();

    // additional check not found in the sdk - this prevents a panic
    if signers.len() != public_keys.len() {
        return Err(AppError::TxValidation(format!(
            "wrong number of signer info; expected {}, got {}",
            signers.len(),
            public_keys.len()
        )));
    }

    for (i, key) in public_keys.into_iter().enumerate() {
        if let Some(key) = key {
            let addr = key.get_address();

            if &addr != signers[i] {
                return Err(AppError::InvalidPublicKey);
            }

            let mut acct = Auth::get_account(ctx, &addr).ok_or(AppError::AccountNotFound)?;

            if acct.get_public_key().is_some() {
                continue;
            }

            acct.set_public_key(key.clone());
            Auth::set_account(ctx, acct)
        }
    }

    Ok(())
}

fn sig_verification_handler<T: DB>(ctx: &mut Context<T>, tx: &DecodedTx) -> Result<(), AppError> {
    let signers = tx.get_signers();
    let signature_data = tx.get_signatures_data();

    // NOTE: this is also checked in validate_basic_ante_handler
    if signature_data.len() != signers.len() {
        return Err(AppError::TxValidation(format!(
            "wrong number of signatures; expected {}, got {}",
            signers.len(),
            signature_data.len()
        )));
    }

    for (i, signature_data) in signature_data.into_iter().enumerate() {
        let signer = signers[i];

        // check sequence number
        let acct = Auth::get_account(ctx, signer).ok_or(AppError::AccountNotFound)?;
        let account_seq = acct.get_sequence();
        if account_seq != signature_data.sequence {
            return Err(AppError::TxValidation(format!(
                "incorrect tx sequence; expected {}, got {}",
                account_seq, signature_data.sequence
            )));
        }

        // check signature
        let sign_bytes = SignDoc {
            body_bytes: tx.tx_raw.body_bytes.clone(),
            auth_info_bytes: tx.tx_raw.auth_info_bytes.clone(),
            chain_id: ctx.get_chain_id().to_owned(),
            account_number: acct.get_account_number(),
        }
        .encode_to_vec();
        let message = secp256k1::Message::from_hashed_data::<sha256::Hash>(&sign_bytes);

        let public_key = acct
            .get_public_key()
            .as_ref()
            .expect("account pub keys are set in set_pub_key_ante_handler");

        //TODO: move sig verification into PublicKey
        match public_key {
            PublicKey::Secp256k1(pub_key) => {
                let public_key =
                    Secp256k1PubKey::from_slice(&Vec::from(pub_key.to_owned())).unwrap(); //TODO: remove unwrap

                let signature = ecdsa::Signature::from_compact(&signature_data.signature).unwrap(); //TODO: remove unwrap

                Secp256k1::verification_only()
                    .verify_ecdsa(&message, &signature, &public_key) //TODO: lib cannot be used for bitcoin sig verification
                    .map_err(|_| {
                        return AppError::TxValidation(format!("invalid signature"));
                    })?;
            }
        }
    }

    return Ok(());
}

fn increment_sequence_ante_handler<T: DB>(
    ctx: &mut Context<T>,
    tx: &DecodedTx,
) -> Result<(), AppError> {
    for signer in tx.get_signers() {
        let mut acct = Auth::get_account(ctx, signer).ok_or(AppError::AccountNotFound)?;
        acct.increment_sequence();
        Auth::set_account(ctx, acct)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use database::MemDB;
    use proto_messages::cosmos::auth::v1beta1::{Account, BaseAccount};
    use proto_types::AccAddress;

    use crate::store::MultiStore;
    use crate::types::tests::get_signed_tx;
    use crate::types::InitContext;

    use super::*;

    #[test]
    fn sig_verification_handler_works() {
        // TODO: add tests for transactions that are expected to fail
        let tx = get_signed_tx();

        let db = MemDB::new();
        let mut store = MultiStore::new(db);
        let mut ctx = InitContext::new(&mut store, 0, "unit-testing".into());
        let account = BaseAccount {
            address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                .unwrap(),
            pub_key: None,
            account_number: 1,
            sequence: 1,
        };
        Auth::set_account(&mut ctx.as_any(), Account::Base(account));
        set_pub_key_ante_handler(&mut ctx.as_any(), &tx).unwrap();
        sig_verification_handler(&mut ctx.as_any(), &tx).unwrap();
    }
}
