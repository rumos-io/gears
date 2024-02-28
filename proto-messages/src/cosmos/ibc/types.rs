//! Module focused on re-export of ibc types and better organization of them
pub mod core {
    pub mod connection {
        pub use ibc::core::connection::types::*;
    }

    pub mod channel {
        pub use ibc::core::channel::types::commitment::*;
        pub mod packet {
            pub use ibc::core::channel::types::packet::*;
        }

        pub mod channel {
            pub use ibc::core::channel::types::channel::*;
        }
    }

    pub mod client {
        pub mod context {
            #[doc(inline)]
            pub use ibc::core::client::context::*;
        }

        pub mod error {
            #[doc(inline)]
            pub use ibc::core::client::types::error::*;
        }

        pub mod query {
            pub use ibc_proto::ibc::core::client::v1::query_client::QueryClient;
        }

        pub mod proto {
            pub use ibc::core::client::types::proto::v1::IdentifiedClientState as RawIdentifiedClientState;
            pub use ibc::core::client::types::proto::v1::Params as RawParams;

            #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            pub struct IdentifiedClientState {
                pub client_id: String,
                pub client_state: Option<crate::any::Any>,
            }

            impl From<RawIdentifiedClientState> for IdentifiedClientState {
                fn from(value: RawIdentifiedClientState) -> Self {
                    let RawIdentifiedClientState {
                        client_id,
                        client_state,
                    } = value;

                    Self {
                        client_id,
                        client_state: client_state.map(crate::any::Any::from),
                    }
                }
            }

            impl From<IdentifiedClientState> for RawIdentifiedClientState {
                fn from(value: IdentifiedClientState) -> Self {
                    let IdentifiedClientState {
                        client_id,
                        client_state,
                    } = value;

                    Self {
                        client_id,
                        client_state: client_state.map(ibc::primitives::proto::Any::from),
                    }
                }
            }
        }

        pub mod types {

            use std::collections::HashSet;

            use serde::{Deserialize, Serialize};

            use crate::any::Any;

            use super::proto::RawParams;
            pub use ibc::core::client::context::types::proto::v1::Height as ProtoHeight;
            pub use ibc::core::client::types::Height as RawHeight;

            #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
            pub struct Height(pub RawHeight);

            impl From<RawHeight> for Height {
                fn from(value: RawHeight) -> Self {
                    Self(value)
                }
            }

            impl From<Height> for RawHeight {
                fn from(value: Height) -> Self {
                    value.0
                }
            }

            impl From<Height> for ProtoHeight {
                fn from(value: Height) -> Self {
                    Self {
                        revision_number: value.0.revision_number(),
                        revision_height: value.0.revision_height(),
                    }
                }
            }

            #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
            #[error("Invalid height")]
            pub struct HeightError;
            impl TryFrom<ProtoHeight> for Height {
                type Error = HeightError;

                fn try_from(value: ProtoHeight) -> Result<Self, Self::Error> {
                    let ProtoHeight {
                        revision_number,
                        revision_height,
                    } = value;

                    Ok(Self(
                        RawHeight::new(revision_number, revision_height)
                            .map_err(|_| HeightError)?,
                    ))
                }
            }

            pub const ALLOW_ALL_CLIENTS: &str = "*";

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            pub struct Params {
                allowed_clients: HashSet<String>,
            }

            impl From<RawParams> for Params {
                fn from(value: RawParams) -> Self {
                    Self {
                        allowed_clients: value.allowed_clients.into_iter().collect(),
                    }
                }
            }

            impl From<Params> for RawParams {
                fn from(value: Params) -> Self {
                    let Params { allowed_clients } = value;

                    Self {
                        allowed_clients: allowed_clients.into_iter().collect(),
                    }
                }
            }

            impl Params {
                pub fn is_client_allowed(
                    &self,
                    client_type: &ibc::core::host::types::identifiers::ClientType,
                ) -> bool {
                    if client_type.as_str().trim().is_empty() {
                        false
                    } else if self.allowed_clients.len() == 1
                        && self.allowed_clients.contains(ALLOW_ALL_CLIENTS)
                    {
                        true
                    } else {
                        self.allowed_clients.contains(client_type.as_str())
                    }
                }
            }

            pub use ibc::core::client::types::proto::v1::ConsensusStateWithHeight as RawConsensusStateWithHeight;

            #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
            pub struct ConsensusStateWithHeight {
                pub height: Option<Height>,
                pub consensus_state: Option<Any>,
            }

            impl From<ConsensusStateWithHeight> for RawConsensusStateWithHeight {
                fn from(value: ConsensusStateWithHeight) -> Self {
                    let ConsensusStateWithHeight {
                        height,
                        consensus_state,
                    } = value;

                    Self {
                        height: height.map(Height::into),
                        consensus_state: consensus_state.map(Any::into),
                    }
                }
            }

            impl TryFrom<RawConsensusStateWithHeight> for ConsensusStateWithHeight {
                type Error = HeightError;

                fn try_from(value: RawConsensusStateWithHeight) -> Result<Self, Self::Error> {
                    let RawConsensusStateWithHeight {
                        height,
                        consensus_state,
                    } = value;

                    let height = if let Some(height) = height {
                        Some(height.try_into()?)
                    } else {
                        None
                    };

                    Ok(Self {
                        height,
                        consensus_state: consensus_state.map(Any::from),
                    })
                }
            }
        }
    }
    pub mod host {
        pub use ibc::core::host::{ExecutionContext, ValidationContext};

        pub mod error {
            #[doc(inline)]
            pub use ibc::core::host::types::error::*;
        }

        pub mod identifiers {
            #[doc(inline)]
            pub use ibc::core::host::types::identifiers::*;
        }

        pub mod path {
            pub use ibc::core::host::types::path::*;
        }
    }

    pub mod handler {
        pub mod error {
            #[doc(inline)]
            pub use ibc::core::handler::types::error::*;
        }
        pub mod events {
            #[doc(inline)]
            pub use ibc::core::handler::types::events::*;
        }
    }

    pub mod commitment {
        #[doc(inline)]
        pub use ibc::core::commitment_types::commitment::*;
    }
}

pub mod primitives {
    pub use ibc::primitives::{Signer, Timestamp};
}

pub mod tendermint {
    pub mod informal {
        #[doc(inline)]
        pub use ::tendermint::informal::abci::*;
    }

    pub mod context {
        #[doc(inline)]
        pub use ibc::clients::tendermint::context::*;
    }
    pub mod consensus_state {
        pub use ibc::clients::tendermint::consensus_state::ConsensusState as WrappedConsensusState;
        pub use ibc::clients::tendermint::types::ConsensusState as RawConsensusState;
    }

    pub use ibc::clients::tendermint::client_state::ClientState as WrappedTendermintClientState;
    pub use ibc::clients::tendermint::types::ClientState as RawTendermintClientState;

    pub mod error {
        pub use ::tendermint::proto::Error;
    }

    pub mod types {

        pub use ibc::clients::tendermint::types::proto::v1::Header as ProtoHeader;
        pub use ibc::clients::tendermint::types::Header as RawHeader;
        use ibc_proto::Protobuf;

        #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
        #[serde(transparent)]
        pub struct Header(pub RawHeader);

        impl Protobuf<ProtoHeader> for Header {}

        impl From<RawHeader> for Header {
            fn from(value: RawHeader) -> Self {
                Self(value)
            }
        }

        impl From<Header> for RawHeader {
            fn from(value: Header) -> Self {
                value.0
            }
        }

        impl TryFrom<ProtoHeader> for Header {
            type Error = ibc::clients::tendermint::types::error::Error;

            fn try_from(value: ProtoHeader) -> Result<Self, Self::Error> {
                Ok(Self(value.try_into()?))
            }
        }

        impl From<Header> for ProtoHeader {
            fn from(value: Header) -> Self {
                value.0.into()
            }
        }
    }
}
