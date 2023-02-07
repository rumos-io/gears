use crate::{
    error::AppError,
    types::{Context, DecodedTx},
    x::auth::Params as AuthParams,
};

pub struct AnteHandler {}

impl AnteHandler {
    pub fn run(ctx: &mut Context, tx: &DecodedTx) -> Result<(), AppError> {
        validate_basic_ante_handler(tx)?;
        tx_timeout_height_ante_handler(ctx, tx)?;
        validate_memo_ante_handler(ctx, tx)?;
        //consume_gas_for_tx_size(ctx, tx)?;

        // ante.NewSetUpContextDecorator(),
        //  - ante.NewRejectExtensionOptionsDecorator(), // Covered in tx parsing code
        //  - NewMempoolFeeDecorator(opts.BypassMinFeeMsgTypes), // NOT USED FOR DELIVER_TX
        //  - ante.NewValidateBasicDecorator(),
        //  - ante.NewTxTimeoutHeightDecorator(),
        //  - ante.NewValidateMemoDecorator(opts.AccountKeeper),
        // ante.NewConsumeGasForTxSizeDecorator(opts.AccountKeeper),
        // ante.NewDeductFeeDecorator(opts.AccountKeeper, opts.BankKeeper, opts.FeegrantKeeper),
        // // SetPubKeyDecorator must be called before all signature verification decorators
        // ante.NewSetPubKeyDecorator(opts.AccountKeeper),
        // ante.NewValidateSigCountDecorator(opts.AccountKeeper),
        // ante.NewSigGasConsumeDecorator(opts.AccountKeeper, sigGasConsumer),
        // ante.NewSigVerificationDecorator(opts.AccountKeeper, opts.SignModeHandler),
        // ante.NewIncrementSequenceDecorator(opts.AccountKeeper),
        // ibcante.NewAnteDecorator(opts.IBCkeeper),

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
            "wrong number of signers; expected {}, got {}",
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

// fn consume_gas_for_tx_size(ctx: &Context, tx: &DecodedTx) -> Result<(), AppError> {
//     let tx_size_cost_per_byte = AuthParams::get(ctx).tx_size_cost_per_byte;

//     //ctx.GasMeter().ConsumeGas(params.TxSizeCostPerByte*sdk.Gas(len(ctx.TxBytes())), "txSize")

//     Ok(())
// }
