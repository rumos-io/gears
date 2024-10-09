use std::marker::PhantomData;

pub mod cli;
pub mod grpc;
pub mod query_handler;
pub mod rest;
pub mod tx_handler;

#[derive(Debug, Clone, Default)]
pub struct GovClientHandler<T>(PhantomData<T>);
