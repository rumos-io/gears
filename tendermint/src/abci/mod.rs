pub mod errors;
pub use tendermint_abci::ServerBuilder;

pub use tendermint_informal::abci::Event;
pub use tendermint_informal::abci::EventAttribute;

pub mod response {
    use bytes::Bytes;
    use serde::{Deserialize, Serialize};
    use tendermint_informal::abci::Code;
    use tendermint_proto::serializers;

    use crate::types::proto::event::Event;

    #[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
    #[serde(default)]
    pub struct DeliverTx {
        /// The response code.
        ///
        /// This code should be `0` only if the transaction is fully valid. However,
        /// invalid transactions included in a block will still be executed against
        /// the application state.
        pub code: Code,
        /// Result bytes, if any.
        #[serde(with = "serializers::nullable")]
        pub data: Bytes,
        /// The output of the application's logger.
        ///
        /// **May be non-deterministic**.
        pub log: String,
        /// Additional information.
        ///
        /// **May be non-deterministic**.
        pub info: String,
        /// Amount of gas requested for the transaction.
        #[serde(with = "crate::types::serializers::from_str")]
        pub gas_wanted: i64,
        /// Amount of gas consumed by the transaction.
        #[serde(with = "crate::types::serializers::from_str")]
        pub gas_used: i64,
        /// Events that occurred while executing the transaction.
        pub events: Vec<Event>,
        /// The namespace for the `code`.
        pub codespace: String,
    }
}

pub use tendermint_abci::cancellation::CancellationSource;
pub use tendermint_abci::cancellation::TokenDropGuard;
