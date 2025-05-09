//!
//! # wRPC Client for Rusty Spectre p2p Node
//!
//! This crate provides a WebSocket RPC client for Rusty Spectre p2p node. It is based on the
//! [wRPC](https://docs.rs/workflow-rpc) crate that offers WebSocket RPC implementation
//! for Rust based on Borsh and Serde JSON serialization. wRPC is a lightweight RPC framework
//! meant to function as an IPC (Inter-Process Communication) mechanism for Rust applications.
//!
//! Rust examples on using wRPC client can be found in the
//! [examples](https://github.com/spectre-project/rusty-spectre/tree/main/rpc/wrpc/examples) folder.
//!
//! WASM bindings for wRPC client can be found in the [`spectre-wrpc-wasm`](https://docs.rs/spectre-wrpc-wasm) crate.
//!
//! The main struct managing Spectre RPC client connections is the [`SpectreRpcClient`].
//!

pub mod client;
pub mod error;
mod imports;
pub mod result;
pub use imports::{Resolver, SpectreRpcClient, WrpcEncoding};
pub mod node;
pub mod parse;
pub mod prelude;
pub mod resolver;
