/*!
# Rusty Spectre WASM32 bindings

[<img alt="github" src="https://img.shields.io/badge/github-spectre--project/rusty--spectre-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/spectre-project/rusty-spectre/tree/main/wasm)
[<img alt="crates.io" src="https://img.shields.io/crates/v/spectre-wasm.svg?maxAge=2592000&style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/spectre-wasm)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-spectre--wasm-56c2a5?maxAge=2592000&style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/spectre-wasm)
<img alt="license" src="https://img.shields.io/crates/l/spectre-wasm.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">

<br>

Rusty-Spectre WASM32 bindings offer direct integration of Rust code and Rusty-Spectre
codebase within JavaScript environments such as Node.js and Web Browsers.

## Documentation

As of now the code is compatible with Kaspa and its documentation can be used from the official links.
Please note that while WASM directly binds JavaScript and Rust resources, their names on JavaScript side
are different from their name in Rust as they conform to the 'camelCase' convention in JavaScript and
to the 'snake_case' convention in Rust.

## Interfaces

The APIs are currently separated into the following groups (this will be expanded in the future):

- **Consensus Client API** — Bindings for primitives related to transactions.
- **RPC API** — [RPC interface bindings](spectre_wrpc_wasm::client) for the Spectre node using WebSocket (wRPC) connections.
- **Wallet SDK** — API for async core wallet processing tasks.
- **Wallet API** — A rust implementation of the fully-featured wallet usable in the native Rust, Browser or NodeJs and Bun environments.

## NPM Modules

For JavaScript / TypeScript environments, there are two
available NPM modules:

- <https://www.npmjs.com/package/spectre>
- <https://www.npmjs.com/package/spectre-wasm>

The `spectre-wasm` module is a pure WASM32 module that includes
the entire wallet framework, but does not support RPC due to an absence
of a native WebSocket in NodeJs environment, while
the `spectre` module includes `websocket` package dependency simulating
the W3C WebSocket and due to this supports RPC.

NOTE: for security reasons it is always recommended to build WASM SDK from source or
download pre-built redistributables from releases or development builds.

## Examples

JavaScript examples for using this framework can be found at:
<https://github.com/spectre-project/rusty-spectre/tree/main/wasm/nodejs>

## WASM32 Binaries

For pre-built browser-compatible WASM32 redistributables of this
framework please see the releases section of the Rusty Spectre
repository at <https://github.com/spectre-project/rusty-spectre/releases>.

## Using RPC

No special handling is required to use the RPC client
in **Browser** or **Bun** environments due to the fact that
these environments provide native WebSocket support.

**NODEJS:** If you are building from source, to use WASM RPC client
in the NodeJS environment, you need to introduce a global W3C WebSocket
object before loading the WASM32 library (to simulate the browser behavior).
You can the [WebSocket](https://www.npmjs.com/package/websocket)
module that offers W3C WebSocket compatibility and is compatible
with Spectre RPC implementation.

You can use the following shims:

```js
// WebSocket
globalThis.WebSocket = require('websocket').w3cwebsocket;
```

## Loading in a Web App

```html
<html>
    <head>
        <script type="module">
            import * as spectre_wasm from './spectre/spectre-wasm.js';
            (async () => {
                const spectre = await spectre_wasm.default('./spectre/spectre-wasm_bg.wasm');
                // ...
            })();
        </script>
    </head>
    <body></body>
</html>
```

## Loading in a Node.js App

```javascript
// W3C WebSocket module shim
// this is provided by NPM `spectre` module and is only needed
// if you are building WASM libraries for NodeJS from source
// globalThis.WebSocket = require('websocket').w3cwebsocket;

let {RpcClient,Encoding,initConsolePanicHook} = require('./spectre-rpc');

// enabling console panic hooks allows WASM to print panic details to console
// initConsolePanicHook();
// enabling browser panic hooks will create a full-page DIV with panic details
// this is useful for mobile devices where console is not available
// initBrowserPanicHook();

// if port is not specified, it will use the default port for the specified network
const rpc = new RpcClient("127.0.0.1", Encoding.Borsh, "testnet-10");
const rpc = new RpcClient({
    url : "127.0.0.1",
    encoding : Encoding.Borsh,
    networkId : "testnet-10"
});


(async () => {
    try {
        await rpc.connect();
        let info = await rpc.getInfo();
        console.log(info);
    } finally {
        await rpc.disconnect();
    }
})();
```

For more details, please follow the integration guide.

*/

#![allow(unused_imports)]

#[cfg(all(
    any(feature = "wasm32-sdk", feature = "wasm32-rpc", feature = "wasm32-core", feature = "wasm32-keygen"),
    not(target_arch = "wasm32")
))]
compile_error!(
    "`spectre-wasm` crate for WASM32 target must be built with `--features wasm32-sdk|wasm32-rpc|wasm32-core|wasm32-keygen`"
);

mod version;
pub use version::*;

cfg_if::cfg_if! {

    if #[cfg(feature = "wasm32-sdk")] {

        pub use spectre_addresses::{Address, Version as AddressVersion};
        pub use spectre_consensus_core::tx::{ScriptPublicKey, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput};
        pub use spectre_pow::wasm::*;
        pub use spectre_txscript::wasm::*;

        pub mod rpc {
            //! Spectre RPC interface
            //!

            pub mod messages {
                //! Spectre RPC messages
                pub use spectre_rpc_core::model::message::*;
            }
            pub use spectre_rpc_core::api::rpc::RpcApi;
            pub use spectre_rpc_core::wasm::message::*;

            pub use spectre_wrpc_wasm::client::*;
            pub use spectre_wrpc_wasm::resolver::*;
            pub use spectre_wrpc_wasm::notify::*;
        }

        pub use spectre_consensus_wasm::*;
        pub use spectre_wallet_keys::prelude::*;
        pub use spectre_wallet_core::wasm::*;

    } else if #[cfg(feature = "wasm32-core")] {

        pub use spectre_addresses::{Address, Version as AddressVersion};
        pub use spectre_consensus_core::tx::{ScriptPublicKey, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput};
        pub use spectre_pow::wasm::*;
        pub use spectre_txscript::wasm::*;

        pub mod rpc {
            //! Spectre RPC interface
            //!

            pub mod messages {
                //! Spectre RPC messages
                pub use spectre_rpc_core::model::message::*;
            }
            pub use spectre_rpc_core::api::rpc::RpcApi;
            pub use spectre_rpc_core::wasm::message::*;

            pub use spectre_wrpc_wasm::client::*;
            pub use spectre_wrpc_wasm::resolver::*;
            pub use spectre_wrpc_wasm::notify::*;
        }

        pub use spectre_consensus_wasm::*;
        pub use spectre_wallet_keys::prelude::*;
        pub use spectre_wallet_core::wasm::*;

    } else if #[cfg(feature = "wasm32-rpc")] {

        pub use spectre_rpc_core::api::rpc::RpcApi;
        pub use spectre_rpc_core::wasm::message::*;
        pub use spectre_rpc_core::wasm::message::IPingRequest;
        pub use spectre_wrpc_wasm::client::*;
        pub use spectre_wrpc_wasm::resolver::*;
        pub use spectre_wrpc_wasm::notify::*;
        pub use spectre_wasm_core::types::*;

    } else if #[cfg(feature = "wasm32-keygen")] {

        pub use spectre_addresses::{Address, Version as AddressVersion};
        pub use spectre_wallet_keys::prelude::*;
        pub use spectre_bip32::*;
        pub use spectre_wasm_core::types::*;

    }
}
