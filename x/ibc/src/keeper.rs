use database::Database;
use gears::{types::context::tx_context::TxContext, x::params::ParamsSubspaceKey};
use prost::Message;
use proto_messages::cosmos::ibc::{
    protobuf::{PrimitiveAny, PrimitiveProtobuf},
    types::{
        core::{
            client::context::{
                client_state::{ClientStateCommon, ClientStateExecution, ClientStateValidation},
                types::{
                    events::{
                        CLIENT_ID_ATTRIBUTE_KEY, CLIENT_MISBEHAVIOUR_EVENT,
                        CLIENT_TYPE_ATTRIBUTE_KEY, CONSENSUS_HEIGHTS_ATTRIBUTE_KEY,
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY, CREATE_CLIENT_EVENT, UPDATE_CLIENT_EVENT,
                        UPGRADE_CLIENT_EVENT,
                    },
                    Status, UpdateKind,
                },
            },
            commitment::{CommitmentProofBytes, CommitmentRoot},
            host::identifiers::{ClientId, ClientType},
        },
        tendermint::{
            consensus_state::WrappedConsensusState, informal::Event, WrappedTendermintClientState,
        },
    },
};
use store::StoreKey;

use crate::{
    errors::{ClientCreateError, ClientUpdateError, ClientUpgradeError, SearchError},
    params::{self, AbciParamsKeeper, Params, RawParams, CLIENT_STATE_KEY},
    types::InitContextShim,
};

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    params_keeper: AbciParamsKeeper<SK, PSK>,
    // auth_keeper: auth::Keeper<SK, PSK>,
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

    pub fn client_upgrade<DB: Database + Send + Sync>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        client_id: &ClientId,
        upgraded_client_state: WrappedTendermintClientState,
        upgraded_consensus_state: WrappedConsensusState,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
    ) -> Result<(), ClientUpgradeError> {
        let client_state = self.client_state_get(ctx)?;

        let mut shim_ctx = InitContextShim(ctx);
        let client_status = client_state.status(&mut shim_ctx, client_id)?;

        if client_status != Status::Active {
            return Err(ClientUpgradeError::NotActive {
                client_id: client_id.clone(),
                status: client_status,
            });
        }

        let last_height = client_state.latest_height();

        {
            let upgraded_height = upgraded_client_state.latest_height();

            if !(upgraded_height > last_height) {
                return Err(ClientUpgradeError::HeightError {
                    upgraded: upgraded_height,
                    current: last_height,
                });
            }
        }

        let root_hash = CommitmentRoot::from_bytes(&self.root_hash(ctx));
        client_state.verify_upgrade_client(
            upgraded_client_state.into(),
            upgraded_consensus_state.into(),
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            &root_hash,
        )?;

        ctx.append_events(vec![
            Event::new(
                UPGRADE_CLIENT_EVENT,
                [
                    (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                    (
                        CLIENT_TYPE_ATTRIBUTE_KEY,
                        client_state.client_type().as_str().to_owned(),
                    ),
                    (
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY,
                        client_state.latest_height().to_string(),
                    ),
                ],
            ),
            Event::new(
                "message",
                [
                    (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                ],
            ),
        ]);

        Ok(())
    }

    pub fn client_update<DB: Database + Send + Sync>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        client_id: &ClientId,
        client_message: PrimitiveAny,
    ) -> Result<(), ClientUpdateError> {
        let client_state = self.client_state_get(ctx)?;
        let client_type = client_state.client_type();
        let params = self.params_get(ctx)?;

        let mut shim_ctx = InitContextShim(ctx);

        let client_status = if params.is_client_allowed(&client_type) {
            Status::Unauthorized
        } else {
            client_state.status(&mut shim_ctx, client_id)?
        };

        if client_status != Status::Active {
            return Err(ClientUpdateError::NotActive {
                client_id: client_id.clone(),
                status: client_status,
            });
        }

        client_state.verify_client_message(
            &shim_ctx,
            client_id,
            client_message.clone(),
            &UpdateKind::UpdateClient,
        )?;
        let misbehaviour = client_state.check_for_misbehaviour(
            &shim_ctx,
            client_id,
            client_message.clone(),
            &UpdateKind::UpdateClient,
        )?;
        if misbehaviour {
            client_state.update_state_on_misbehaviour(
                &mut shim_ctx,
                client_id,
                client_message,
                &UpdateKind::UpdateClient,
            )?;

            ctx.append_events(vec![
                Event::new(
                    CLIENT_MISBEHAVIOUR_EVENT,
                    [
                        (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                        (CLIENT_TYPE_ATTRIBUTE_KEY, client_type.as_str().to_owned()),
                    ],
                ),
                Event::new(
                    "message",
                    [
                        (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                    ],
                ),
            ]);

            Ok(())
        } else {
            let height = client_state.update_state(&mut shim_ctx, client_id, client_message)?;
            let consensus_state_height = height
                .first()
                .map(|this| this.to_string())
                .unwrap_or(String::new());

            let mut heights = String::new();
            height
                .into_iter()
                .for_each(|this| heights.push_str(&this.to_string()));

            ctx.append_events(vec![
                Event::new(
                    UPDATE_CLIENT_EVENT,
                    [
                        (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                        (CLIENT_TYPE_ATTRIBUTE_KEY, client_type.as_str().to_owned()),
                        (CONSENSUS_HEIGHT_ATTRIBUTE_KEY, consensus_state_height), // NOTE: Deprecated: so it should be removed
                        (CONSENSUS_HEIGHTS_ATTRIBUTE_KEY, heights),
                    ],
                ),
                Event::new(
                    "message",
                    [
                        (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                    ],
                ),
            ]);

            Ok(())
        }
    }

    pub fn client_create<'a, 'b, DB: Database + Send + Sync>(
        &mut self,
        ctx: &'a mut TxContext<'b, DB, SK>,
        client_state: &(impl ClientStateCommon
              + ClientStateExecution<InitContextShim<'a, 'b, DB, SK>>
              + ClientStateValidation<InitContextShim<'a, 'b, DB, SK>>),
        consensus_state: WrappedConsensusState,
    ) -> Result<ClientId, ClientCreateError> {
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

        // TODO: Is this okay to create events before rest of code?
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
                    (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                ],
            ),
        ]);

        {
            // FIXME: fix lifetimes so borrow checker would be happy with this code before events
            let mut ctx = InitContextShim(ctx);

            client_state.initialise(&mut ctx, &client_id, consensus_state.into())?;
            client_state.status(&mut ctx, &client_id)?;
        }

        Ok(client_id)
    }

    fn client_indentifier_generate<DB: Database>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        client_type: &ClientType,
    ) -> Result<ClientId, ClientCreateError> {
        let next_client_seq = self.next_client_sequence_get(ctx)?;

        self.next_client_sequence_set(ctx, next_client_seq + 1);

        ClientId::new(client_type.as_str(), next_client_seq)
            .map_err(ClientCreateError::IdentifierError)
    }

    fn next_client_sequence_set<DB: Database>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        sequence: u64,
    ) {
        let mut ctx = gears::types::context::context::Context::TxContext(ctx);
        self.params_keeper.set(
            &mut ctx,
            params::NEXT_CLIENT_SEQUENCE.as_bytes().into_iter().cloned(),
            sequence.to_be_bytes(),
        )
    }

    fn next_client_sequence_get<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<u64, ClientCreateError> {
        let ctx = gears::types::context::context::Context::TxContext(ctx);
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
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<Params, SearchError> {
        let ctx = gears::types::context::context::Context::TxContext(ctx);
        let bytes = self
            .params_keeper
            .get(&ctx, &params::CLIENT_PARAMS_KEY)
            .map_err(|_| SearchError::NotFound)?;

        Ok(RawParams::decode(bytes.as_slice())
            .map_err(|e| SearchError::DecodeError(e.to_string()))?
            .into())
    }

    fn client_state_get<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<WrappedTendermintClientState, SearchError> {
        // TODO: Unsure in this code https://github.com/cosmos/ibc-go/blob/41e7bf14f717d5cc2815688c8c590769ed164389/modules/core/02-client/keeper/keeper.go#L78
        let store = ctx.get_kv_store(&self.store_key);
        let bytes = store
            .get(CLIENT_STATE_KEY.as_bytes())
            .ok_or(SearchError::NotFound)?;
        let state =
            <WrappedTendermintClientState as PrimitiveProtobuf<PrimitiveAny>>::decode_vec(&bytes)
                .map_err(|e| SearchError::DecodeError(e.to_string()))?;

        Ok(state)
    }

    fn root_hash<DB: Database>(&self, ctx: &mut TxContext<'_, DB, SK>) -> [u8; 32] {
        ctx.get_kv_store(&self.store_key).head_commit_hash()
    }
}
