//!
//! Spectre Wallet Core - Multi-platform Rust framework for Spectre Wallet.
//!
//! This framework provides a series of APIs and primitives
//! to simplify building applications that interface with
//! the Spectre p2p network.
//!
//! Included are low-level primitives
//! such as [`UtxoProcessor`](crate::utxo::UtxoProcessor)
//! and [`UtxoContext`](crate::utxo::UtxoContext) that provide
//! various levels of automation as well as higher-level
//! APIs such as [`Wallet`](crate::wallet::Wallet),
//! [`Account`](crate::account::Account) (managed via
//! [`WalletApi`](crate::api::WalletApi) trait)
//! that offer a fully-featured wallet implementation
//! backed by a multi-platform data storage layer capable of
//! storing wallet data on a local file-system as well as
//! within the browser environment.
//!
//! The wallet framework also includes transaction
//! [`Generator`](crate::tx::generator::Generator)
//! that can be used to generate transactions from a set of
//! UTXO entries. The generator can be used to create
//! simple transactions as well as batch transactions
//! comprised of multiple chained transactions.  Batch
//! transactions (also known as compound transactions)
//! are needed when the total number of inputs required
//! to satisfy the requested amount exceeds the maximum
//! allowed transaction mass.
//!
//! The framework can operate
//! within native Rust applications as well as within the NodeJS
//! and browser environments via WASM32.
//!
//! For JavaScript / TypeScript environments, there are two
//! available NPM modules:
//! - <https://www.npmjs.com/package/spectre>
//! - <https://www.npmjs.com/package/spectre-wasm>
//!
//! The `spectre-wasm` module is a pure WASM32 module that includes
//! the entire wallet framework, but does not support RPC due to an absence
//! of a native WebSocket in NodeJs environment, while
//! the `spectre` module includes `websocket` module dependency simulating
//! the W3C WebSocket and thus supports RPC.
//!
//! JavaScript examples for using this framework can be found at:
//! <https://github.com/spectre-project/rusty-spectre/tree/main/wasm/nodejs>
//!
//! For pre-built browser-compatible WASM32 redistributables of this
//! framework please see the releases section of the Rusty Spectre
//! repository at <https://github.com/spectre-project/rusty-spectre/releases>.
//!

extern crate alloc;
extern crate self as spectre_wallet_core;

// use cfg_if::cfg_if;

// cfg_if! {
//     if #[cfg(feature = "wasm32-core")] {
//         // pub mod wasm;
//         // pub use wasm::*;

//         pub mod account;
//         pub mod api;
//         pub mod compat;
//         pub mod derivation;
//         pub mod deterministic;
//         pub mod encryption;
//         pub mod error;
//         pub mod events;
//         pub mod factory;
//         mod imports;
//         pub mod message;
//         pub mod prelude;
//         pub mod result;
//         pub mod rpc;
//         pub mod serializer;
//         pub mod settings;
//         pub mod storage;
//         pub mod tx;
//         pub mod utils;
//         pub mod utxo;
//         pub mod wallet;

//     } else if #[cfg(any(feature = "wasm32-sdk", not(target_arch = "wasm32")))] {
pub mod account;
pub mod api;
pub mod compat;
pub mod cryptobox;
pub mod derivation;
pub mod deterministic;
pub mod encryption;
pub mod error;
pub mod events;
pub mod factory;
mod imports;
pub mod message;
pub mod metrics;
pub mod prelude;
pub mod result;
pub mod rpc;
pub mod serializer;
pub mod settings;
pub mod storage;
pub mod tx;
pub mod utils;
pub mod utxo;
pub mod wallet;
//     }

// }

#[cfg(any(feature = "wasm32-sdk", feature = "wasm32-core"))]
pub mod wasm;

/// Returns the version of the Wallet framework.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns the version of the Wallet framework combined with short git hash.
pub fn version_with_git_hash() -> String {
    spectre_utils::git::with_short_hash(env!("CARGO_PKG_VERSION")).to_string()
}

#[cfg(test)]
pub mod tests;
