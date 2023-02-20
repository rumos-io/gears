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
    pub fn run(ctx: &mut Context, tx: &DecodedTx) -> Result<(), AppError> {
        validate_basic_ante_handler(tx)?;
        tx_timeout_height_ante_handler(ctx, tx)?;
        validate_memo_ante_handler(ctx, tx)?;
        deduct_fee_ante_handler(ctx, tx)?;
        set_pub_key_ante_handler(ctx, tx)?;
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
        //  ** ante.NewSigVerificationDecorator(opts.AccountKeeper, opts.SignModeHandler),
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

fn tx_timeout_height_ante_handler(ctx: &Context, tx: &DecodedTx) -> Result<(), AppError> {
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

fn validate_memo_ante_handler(ctx: &Context, tx: &DecodedTx) -> Result<(), AppError> {
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

fn deduct_fee_ante_handler(ctx: &mut Context, tx: &DecodedTx) -> Result<(), AppError> {
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

fn set_pub_key_ante_handler(ctx: &mut Context, tx: &DecodedTx) -> Result<(), AppError> {
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

    for (i, key) in public_keys.iter().enumerate() {
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

fn increment_sequence_ante_handler(ctx: &mut Context, tx: &DecodedTx) -> Result<(), AppError> {
    for signer in tx.get_signers() {
        let mut acct = Auth::get_account(ctx, signer).ok_or(AppError::AccountNotFound)?;
        acct.increment_sequence();
        Auth::set_account(ctx, acct)
    }

    Ok(())
}
