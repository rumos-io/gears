pub mod block {
    pub use tendermint_rpc::endpoint::block::Response;
}

pub mod tx {
    pub use tendermint_rpc::endpoint::tx::Response;

    pub mod broadcast {
        pub use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
        pub use tendermint_rpc::endpoint::broadcast::tx_sync::Response as SyncResponse;
    }

    pub mod search {
        pub use tendermint_rpc::endpoint::tx_search::Response;
    }
}

pub mod validators {
    pub use tendermint_rpc::endpoint::validators::Response;
}
