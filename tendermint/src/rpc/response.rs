pub mod tx {
    pub mod broadcast {
        pub use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
    }

    pub mod search {
        pub use tendermint_rpc::endpoint::tx_search::Response;
    }
}
