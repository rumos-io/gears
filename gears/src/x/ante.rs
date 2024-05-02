use crate::application::handlers::node::AnteHandlerTrait;
use crate::crypto::keys::ReadAccAddress;
use crate::crypto::public::PublicKey;
use crate::signing::handler::MetadataGetter;
use crate::signing::{handler::SignModeHandler, renderer::value_renderer::ValueRenderer};
use crate::types::auth::gas::Gas;
use crate::types::context::tx::TxContext;
use crate::types::context::QueryableContext;
use crate::types::denom::Denom;
use crate::types::gas::descriptor::{ANTE_SECKP251K1_DESCRIPTOR, TX_SIZE_DESCRIPTOR};
use crate::types::gas::kind::TxKind;
use crate::types::gas::{GasErrors, GasMeter};
use crate::x::keepers::auth::AuthKeeper;
use crate::x::keepers::auth::AuthParams;
use crate::x::keepers::bank::BankKeeper;
use crate::x::module::Module;
use crate::{
    error::AppError,
    types::{
        context::TransactionalContext,
        tx::{data::TxData, raw::TxWithRaw, signer::SignerData, Tx, TxMessage},
    },
};
use core_types::tx::signature::SignatureData;
use core_types::{
    signing::SignDoc,
    tx::mode_info::{ModeInfo, SignMode},
};
use prost::Message as ProstMessage;
use std::marker::PhantomData;
use store_crate::database::Database;
use store_crate::StoreKey;

pub trait SignGasConsumer: Clone + Sync + Send + 'static {
    fn consume<AP: AuthParams>(
        &self,
        gas_meter: &mut GasMeter<TxKind>,
        pub_key: PublicKey,
        data: &SignatureData,
        params: &AP,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct DefaultSignGasConsumer;

impl SignGasConsumer for DefaultSignGasConsumer {
    fn consume<AP: AuthParams>(
        &self,
        gas_meter: &mut GasMeter<TxKind>,
        pub_key: PublicKey,
        _data: &SignatureData,
        params: &AP,
    ) -> anyhow::Result<()> {
        // TODO I'm unsure that this is 100% correct due multisig mode see: https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/auth/ante/sigverify.go#L401
        match pub_key {
            PublicKey::Secp256k1(_key) => {
                let amount = params.sig_verify_cost_secp256k1().try_into()?;
                gas_meter.consume_gas(amount, ANTE_SECKP251K1_DESCRIPTOR)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BaseAnteHandler<BK: BankKeeper<SK>, AK: AuthKeeper<SK>, SK: StoreKey, GC> {
    bank_keeper: BK,
    auth_keeper: AK,
    sign_gas_consumer: GC,
    sk: PhantomData<SK>,
}

impl<SK, BK, AK, GC: SignGasConsumer> AnteHandlerTrait<SK> for BaseAnteHandler<BK, AK, SK, GC>
where
    SK: StoreKey,
    BK: BankKeeper<SK>,
    AK: AuthKeeper<SK>,
    GC: SignGasConsumer,
{
    fn run<DB: Database, M: TxMessage + ValueRenderer>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError> {
        BaseAnteHandler::run(self, ctx, tx)
    }
}

impl<AK: AuthKeeper<SK>, BK: BankKeeper<SK>, SK: StoreKey, GC: SignGasConsumer>
    BaseAnteHandler<BK, AK, SK, GC>
{
    pub fn new(
        auth_keeper: AK,
        bank_keeper: BK,
        sign_gas_consumer: GC,
    ) -> BaseAnteHandler<BK, AK, SK, GC> {
        BaseAnteHandler {
            bank_keeper,
            auth_keeper,
            sign_gas_consumer,
            sk: PhantomData,
        }
    }
    pub fn run<DB: Database, M: TxMessage + ValueRenderer>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError> {
        // Note: we currently don't have simulate mode at all, so some methods receive hardcoded values for this mode
        // ante.NewSetUpContextDecorator(), // WE not going to implement this in ante. Some logic should be in application
        self.validate_basic_ante_handler(&tx.tx)?;
        self.tx_timeout_height_ante_handler(ctx, &tx.tx)?;
        self.validate_memo_ante_handler(ctx, &tx.tx)?;
        self.consume_gas_for_tx_size(ctx, tx)
            .map_err(|e| AppError::Custom(e.to_string()))?;
        self.deduct_fee_ante_handler(ctx, &tx.tx)?;
        self.set_pub_key_ante_handler(ctx, &tx.tx)?;
        //  ** ante.NewValidateSigCountDecorator(opts.AccountKeeper),
        self.sign_gas_consume(ctx, &tx.tx)
            .map_err(|e| AppError::Custom(e.to_string()))?;
        self.sig_verification_handler(ctx, tx)?;
        self.increment_sequence_ante_handler(ctx, &tx.tx)?;
        //  ** ibcante.NewAnteDecorator(opts.IBCkeeper),

        //  - ante.NewRejectExtensionOptionsDecorator(), // Covered in tx parsing code
        //  - NewMempoolFeeDecorator(opts.BypassMinFeeMsgTypes), // NOT USED FOR DELIVER_TX
        //  - ante.NewValidateBasicDecorator(),
        //  - ante.NewTxTimeoutHeightDecorator(),
        //  - ante.NewValidateMemoDecorator(opts.AccountKeeper),
        //  - ante.NewConsumeGasForTxSizeDecorator(opts.AccountKeeper),
        //  - ante.NewDeductFeeDecorator(opts.AccountKeeper, opts.BankKeeper, opts.FeegrantKeeper),
        // // SetPubKeyDecorator must be called before all signature verification decorators
        //  - ante.NewSetPubKeyDecorator(opts.AccountKeeper),
        //  ** ante.NewValidateSigCountDecorator(opts.AccountKeeper),
        //  ante.NewSigGasConsumeDecorator(opts.AccountKeeper, sigGasConsumer),
        //  - ante.NewSigVerificationDecorator(opts.AccountKeeper, opts.SignModeHandler),
        //  - ante.NewIncrementSequenceDecorator(opts.AccountKeeper),
        //  ** ibcante.NewAnteDecorator(opts.IBCkeeper),

        Ok(())
    }

    fn consume_gas_for_tx_size<M: TxMessage, DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        TxWithRaw {
            tx: _,
            raw: _,
            tx_len,
        }: &TxWithRaw<M>,
    ) -> anyhow::Result<()> {
        let params = self.auth_keeper.get_auth_params(ctx);
        let tx_len: Gas = (*tx_len as u64).try_into()?;
        let cost_per_byte: Gas = params.tx_cost_per_byte().try_into()?;
        let gas_required = tx_len
            .checked_mul(cost_per_byte)
            .ok_or(GasErrors::ErrorGasOverflow("tx size".to_string()))?;

        ctx.gas_meter
            .consume_gas(gas_required, TX_SIZE_DESCRIPTOR)?;

        Ok(())
    }

    fn sign_gas_consume<M: TxMessage, DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        tx: &Tx<M>,
    ) -> anyhow::Result<()> {
        let auth_params = self.auth_keeper.get_auth_params(ctx);

        let signatures = tx.get_signatures_data();
        let signers_addr = tx.get_signers();

        for (i, signer_addr) in signers_addr.into_iter().enumerate() {
            let acct = self
                .auth_keeper
                .get_account(ctx, signer_addr)
                .ok_or(AppError::AccountNotFound)?;

            let pub_key = acct
                .get_public_key()
                .expect("account pub keys are set in set_pub_key_ante_handler")
                .to_owned();

            let sig = signatures.get(i).expect("TODO");

            self.sign_gas_consumer
                .consume(&mut ctx.gas_meter, pub_key, sig, &auth_params)?;
        }

        Ok(())
    }

    fn validate_basic_ante_handler<M: TxMessage>(&self, tx: &Tx<M>) -> Result<(), AppError> {
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

        Ok(())
    }

    fn tx_timeout_height_ante_handler<DB: Database, CTX: QueryableContext<DB, SK>, M: TxMessage>(
        &self,
        ctx: &CTX,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        let timeout_height = tx.get_timeout_height();

        // timeout_height of zero means no timeout height
        if timeout_height == 0 {
            return Ok(());
        }

        let block_height = ctx.height();

        if ctx.height() > timeout_height {
            return Err(AppError::Timeout {
                timeout: timeout_height,
                current: block_height,
            });
        }

        Ok(())
    }

    fn validate_memo_ante_handler<DB: Database, CTX: QueryableContext<DB, SK>, M: TxMessage>(
        &self,
        ctx: &CTX,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        let max_memo_chars = self.auth_keeper.get_auth_params(ctx).max_memo_characters();
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

    fn deduct_fee_ante_handler<
        'a,
        DB: Database,
        CTX: TransactionalContext<DB, SK>,
        M: TxMessage,
    >(
        &self,
        ctx: &mut CTX,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        let fee = tx.get_fee();
        let fee_payer = tx.get_fee_payer();

        if !self.auth_keeper.has_account(ctx, fee_payer) {
            return Err(AppError::AccountNotFound);
        }

        if let Some(fee) = fee {
            self.bank_keeper.send_coins_from_account_to_module(
                ctx,
                fee_payer.to_owned(),
                Module::FeeCollector,
                fee.to_owned(),
            )?;
        }

        Ok(())
    }

    fn set_pub_key_ante_handler<DB: Database, CTX: TransactionalContext<DB, SK>, M: TxMessage>(
        &self,
        ctx: &mut CTX,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
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

                let mut acct = self
                    .auth_keeper
                    .get_account(ctx, &addr)
                    .ok_or(AppError::AccountNotFound)?;

                if acct.get_public_key().is_some() {
                    continue;
                }

                acct.set_public_key(key.clone());
                self.auth_keeper.set_account(ctx, acct)
            }
        }

        Ok(())
    }

    fn sig_verification_handler<
        DB: Database,
        CTX: TransactionalContext<DB, SK>,
        M: TxMessage + ValueRenderer,
    >(
        &self,
        ctx: &mut CTX,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError> {
        let signers = tx.tx.get_signers();
        let signature_data = tx.tx.get_signatures_data();

        // NOTE: this is also checked in validate_basic_ante_handler
        if signature_data.len() != signers.len() {
            return Err(AppError::TxValidation(format!(
                "wrong number of signatures; expected {}, got {}",
                signers.len(),
                signature_data.len()
            )));
        }

        for (i, signature_data) in signature_data.iter().enumerate() {
            let signer = signers[i];

            // check sequence number
            let acct = self
                .auth_keeper
                .get_account(ctx, signer)
                .ok_or(AppError::AccountNotFound)?;
            let account_seq = acct.get_sequence();
            if account_seq != signature_data.sequence {
                return Err(AppError::TxValidation(format!(
                    "incorrect tx sequence; expected {}, got {}",
                    account_seq, signature_data.sequence
                )));
            }

            let public_key = acct
                .get_public_key()
                .expect("account pub keys are set in set_pub_key_ante_handler"); //TODO: but can't they be set to None?

            let sign_bytes = match &signature_data.mode_info {
                ModeInfo::Single(mode) => match mode {
                    SignMode::Direct => SignDoc {
                        body_bytes: tx.raw.body_bytes.clone(),
                        auth_info_bytes: tx.raw.auth_info_bytes.clone(),
                        chain_id: ctx.chain_id().to_string(),
                        account_number: acct.get_account_number(),
                    }
                    .encode_to_vec(),
                    SignMode::Textual => {
                        let handler = SignModeHandler;

                        let signer_data = SignerData {
                            address: signer.to_owned(),
                            chain_id: ctx.chain_id().to_owned(),
                            account_number: acct.get_account_number(),
                            sequence: account_seq,
                            pub_key: public_key.to_owned(),
                        };

                        let tx_data = TxData {
                            body: tx.tx.body.clone(),
                            auth_info: tx.tx.auth_info.clone(),
                        };

                        let f = MetadataFromState {
                            bank_keeper: &self.bank_keeper,
                            ctx,
                            _phantom: PhantomData,
                        };

                        handler.sign_bytes_get(&f, signer_data, tx_data).unwrap()
                        //TODO: remove unwrap
                    }
                    _ => {
                        return Err(AppError::TxValidation(
                            "sign mode not supported".to_string(),
                        ))
                    }
                },
                ModeInfo::Multi(_) => {
                    return Err(AppError::TxValidation(
                        "multi sig not supported".to_string(),
                    ));
                }
            };

            public_key
                .verify_signature(&sign_bytes, &signature_data.signature)
                .map_err(|e| AppError::TxValidation(format!("invalid signature: {}", e)))?;
        }

        Ok(())
    }

    fn increment_sequence_ante_handler<
        DB: Database,
        CTX: TransactionalContext<DB, SK>,
        M: TxMessage,
    >(
        &self,
        ctx: &mut CTX,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        for signer in tx.get_signers() {
            let mut acct = self
                .auth_keeper
                .get_account(ctx, signer)
                .ok_or(AppError::AccountNotFound)?;
            acct.increment_sequence();
            self.auth_keeper.set_account(ctx, acct)
        }

        Ok(())
    }
}

pub struct MetadataFromState<'a, DB, SK, BK, CTX> {
    pub bank_keeper: &'a BK,
    pub ctx: &'a CTX,
    pub _phantom: PhantomData<(DB, SK)>,
}

impl<'a, DB: Database, SK: StoreKey, BK: BankKeeper<SK>, CTX: TransactionalContext<DB, SK>>
    MetadataGetter for MetadataFromState<'a, DB, SK, BK, CTX>
{
    type Error = std::io::Error; // this is not used here

    fn metadata(
        &self,
        denom: &Denom,
    ) -> Result<Option<crate::types::tx::metadata::Metadata>, Self::Error> {
        Ok(self.bank_keeper.get_denom_metadata(self.ctx, denom))
    }
}

// TODO: uncomment tests
// #[cfg(test)]
// mod tests {
//     use database::MemDB;
//     use proto_messages::cosmos::auth::v1beta1::{Account, BaseAccount};
//     use proto_types::AccAddress;

//     use crate::store::MultiStore;
//     use crate::types::tests::get_signed_tx;
//     use crate::types::InitContext;
//     use crate::x::auth::Auth;

//     use super::*;

//     #[test]
//     fn sig_verification_handler_works() {
//         // TODO: add tests for transactions that are expected to fail
//         let tx = get_signed_tx();

//         let db = MemDB::new();
//         let mut store = MultiStore::new(db);
//         let mut ctx = InitContext::new(&mut store, 0, "unit-testing".into());
//         let account = BaseAccount {
//             address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
//                 .unwrap(),
//             pub_key: None,
//             account_number: 1,
//             sequence: 1,
//         };
//         Auth::set_account(&mut ctx.as_any(), Account::Base(account));
//         set_pub_key_ante_handler(&mut ctx.as_any(), &tx).unwrap();
//         sig_verification_handler(&mut ctx.as_any(), &tx).unwrap();
//     }
// }
