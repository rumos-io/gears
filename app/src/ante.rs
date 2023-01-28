use crate::{
    error::AppError,
    types::{Context, DecodedTx},
};

pub struct AnteHandler {}

impl AnteHandler {
    pub fn run(ctx: &mut Context, tx: &DecodedTx) -> Result<(), AppError> {
        // ante.NewSetUpContextDecorator(),
        // ante.NewRejectExtensionOptionsDecorator(),
        // NewMempoolFeeDecorator(opts.BypassMinFeeMsgTypes),
        // ante.NewValidateBasicDecorator(),
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
