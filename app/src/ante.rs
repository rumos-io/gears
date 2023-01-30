use crate::{
    error::AppError,
    types::{Context, DecodedTx},
};

pub struct AnteHandler {}

impl AnteHandler {
    pub fn run(ctx: &mut Context, tx: &DecodedTx) -> Result<(), AppError> {
        validate_basic_ante_handler(tx)?;
        // ante.NewSetUpContextDecorator(),
        //  - ante.NewRejectExtensionOptionsDecorator(), // Covered in tx parsing code
        //  - NewMempoolFeeDecorator(opts.BypassMinFeeMsgTypes), // NOT USED FOR DELIVER_TX
        //  - ante.NewValidateBasicDecorator(),
        // ante.NewTxTimeoutHeightDecorator(),
        // ante.NewValidateMemoDecorator(opts.AccountKeeper),
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
