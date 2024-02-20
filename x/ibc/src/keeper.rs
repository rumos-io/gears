use database::Database;
use gears::{types::context::init_context::InitContext, x::params::ParamsSubspaceKey};
use prost::Message;
use proto_messages::cosmos::ibc::{
    tx::MsgCreateClient,
    types::{
        client_state::{ClientStateCommon, ClientStateExecution, ClientStateValidation},
        types::events::{
            CLIENT_ID_ATTRIBUTE_KEY, CLIENT_TYPE_ATTRIBUTE_KEY, CONSENSUS_HEIGHT_ATTRIBUTE_KEY,
            CREATE_CLIENT_EVENT,
        },
        ClientError, ClientExecutionContext, ClientType, Event, EventAttribute, IdentifierError,
        RawClientId,
    },
};
use store::StoreKey;

use crate::{
    params::{self, AbciParamsKeeper, Params, ParamsError, RawParams},
    types::{ClientStore, InitContextShim},
};

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    params_keeper: AbciParamsKeeper<SK, PSK>,
    // auth_keeper: auth::Keeper<SK, PSK>,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientCreateError {
    #[error("cannot create client of type: {0}")]
    InvalidType(ClientType),
    #[error("client state type {0} is not registered in the allowlist")]
    NotAllowed(ClientType),
    #[error("{0}")]
    ParamsError(#[from] ParamsError),
    #[error("Decode error: {0}")]
    DecodeError(#[from] prost::DecodeError),
    #[error("{0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("Unexpected error: {0}")]
    CustomError(String),
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::x::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let abci_params_keeper = AbciParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        Keeper {
            store_key,
            params_keeper: abci_params_keeper,
        }
    }

    pub fn client_create<DB: Database + Send + Sync, E: ClientExecutionContext>(
        &mut self,
        ctx: &mut InitContext<'_, DB, SK>,
        msg: MsgCreateClient,
        // client_state: impl ClientStateCommon + ClientStateExecution<E>,
        // consensus_state: RawConsensusState,
    ) -> Result<RawClientId, ClientCreateError> {
        let MsgCreateClient {
            client_state,
            consensus_state,
            signer: _signer, // TODO: is it okay to ignore this field?
        } = msg;

        let client_type = client_state.client_type();
        if client_type
            == ClientType::new("09-localhost")
                .expect("Unreachable: localhost should be valid client type")
        {
            return Err(ClientCreateError::InvalidType(client_state.client_type()));
        }

        let params = self.params_get(ctx)?;

        if !params.is_client_allowed(&client_type) {
            return Err(ClientCreateError::NotAllowed(client_type));
        }

        let client_id = self.client_indentifier_generate(ctx, &client_type)?;

        // let client_store = ClientStore(ctx.get_mutable_kv_store(&self.store_key));
        // {
        //     let mut ctx = InitContextShim(ctx);

        //     client_store.initialise(&mut ctx, &client_id, consensus_state.into())?;
        //     client_store.status(&mut ctx, &client_id);
        // }

        ctx.append_events(vec![
            Event::new(
                CREATE_CLIENT_EVENT,
                [
                    (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                    (CLIENT_TYPE_ATTRIBUTE_KEY, client_type.as_str().to_owned()),
                    (
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY,
                        client_state.latest_height().to_string(),
                    ),
                ],
            ),
            Event::new(
                "message",
                [
                    (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO
                ],
            ),
        ]);

        Ok(client_id)
    }

    fn client_indentifier_generate<DB: Database>(
        &mut self,
        ctx: &mut InitContext<'_, DB, SK>,
        client_type: &ClientType,
    ) -> Result<RawClientId, ClientCreateError> {
        let next_client_seq = self.next_client_sequence_get(ctx)?;

        self.next_client_sequence_set(ctx, next_client_seq + 1);

        RawClientId::new(client_type.as_str(), next_client_seq)
            .map_err(ClientCreateError::IdentifierError)
    }

    fn next_client_sequence_set<DB: Database>(
        &mut self,
        ctx: &mut InitContext<'_, DB, SK>,
        sequence: u64,
    ) {
        let mut ctx = gears::types::context::context::Context::InitContext(ctx);
        self.params_keeper.set(
            &mut ctx,
            params::NEXT_CLIENT_SEQUENCE.as_bytes().into_iter().cloned(),
            sequence.to_be_bytes(),
        )
    }

    fn next_client_sequence_get<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
    ) -> Result<u64, ClientCreateError> {
        let ctx = gears::types::context::context::Context::InitContext(ctx);
        let bytes = self
            .params_keeper
            .get(&ctx, &params::NEXT_CLIENT_SEQUENCE)?;

        if bytes.is_empty() {
            Err(ClientCreateError::CustomError(
                "next client sequence is empty".to_owned(),
            ))?
        }

        // TODO: should return error if remains != empty or ignore?
        let (int_bytes, _remains) = bytes.split_at(std::mem::size_of::<u64>());

        Ok(u64::from_be_bytes(int_bytes.try_into().map_err(
            |e: std::array::TryFromSliceError| ClientCreateError::CustomError(e.to_string()),
        )?))
    }

    fn params_get<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
    ) -> Result<Params, ClientCreateError> {
        let ctx = gears::types::context::context::Context::InitContext(ctx);
        let bytes = self.params_keeper.get(&ctx, &params::CLIENT_PARAMS_KEY)?;

        Ok(RawParams::decode(bytes.as_slice())?.into())
    }
}
