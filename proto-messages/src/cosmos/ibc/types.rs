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
}
