# Spectre On Rust

[![Build Status](https://github.com/spectre-project/rusty-spectre/actions/workflows/ci.yaml/badge.svg)](https://github.com/spectre-project/rusty-spectre/actions/workflows/ci.yaml)
[![GitHub release](https://img.shields.io/github/v/release/spectre-project/rusty-spectre.svg)](https://github.com/spectre-project/rusty-spectre/releases)
[![GitHub license](https://img.shields.io/github/license/spectre-project/rusty-spectre.svg)](https://github.com/spectre-project/rusty-spectre/blob/main/LICENSE)
[![GitHub downloads](https://img.shields.io/github/downloads/spectre-project/rusty-spectre/total.svg)](https://github.com/spectre-project/rusty-spectre/releases)
[![Join the Spectre Discord Server](https://img.shields.io/discord/1233113243741061240.svg?label=&logo=discord&logoColor=ffffff&color=5865F2)](https://discord.com/invite/FZPYpwszcF)

Welcome to the Rust-based implementation of the Spectre full-node and
its ancillary libraries. The contained node release serves as a
drop-in replacement to the established [Golang node](https://github.com/spectre-project/spectred)
and to date is the recommended node software for the Spectre network,
introducing developers to the possibilities of Rust in the Spectre
network's context.

We invite developers and blockchain enthusiasts to collaborate, test,
and optimize our Rust implementation. Each line of code here is an
opportunity to contribute to the open-source blockchain movement,
shaping a platform designed for scalability and speed without
compromising on decentralization.

Your feedback, contributions, and issue reports will be integral to
evolving this codebase and continuing its maturity as a reliable node
in the Spectre network.

## Overview

Spectre on Rust is a fork of [Kaspa on Rust](https://github.com/kaspanet/rusty-kaspa)
introducing CPU-only mining algorithm [SpectreX](https://github.com/spectre-project/rusty-spectrex).

SpectreX is based on [AstroBWTv3](https://github.com/deroproject/derohe/tree/main/astrobwt/astrobwtv3)
and proof-of-work calculation is done in the following steps:

* Step 1: SHA-3
* Step 2: AstroBWTv3
* Step 3: HeavyHash

Spectre will add full non-disclosable privacy and anonymous
transactions in future implemented with the GhostFACE protocol
build by a team of anonymous crypto algorithm researchers and
engineers. Simple and plain goal:

* PHANTOM Protocol + GhostDAG + GhostFACE = Spectre

Spectre will become a ghostchain; nothing more, nothing less. Design
decisions have been made already and more details about the GhostFACE
protocol will be released at a later stage. Sneak peak: It will use
[Pedersen Commitments](https://github.com/dalek-cryptography/bulletproofs)
as it allows perfect integration with the Spectre UTXO model and
allows perfect hiding. ElGamal will be used for TX signature signing
as it has a superior TPS (transactions per second) performance. Any PRs
are welcome and can be made with anonymous accounts. No pre-mine, no
shit, pure privacy is a hit!

## Comparison

Why another fork? Kaspa is great but we love privacy, Monero and DERO
are great but we love speed! So lets join the cool things from both.
We decided to take Kaspa as codebase, quick comparison:

Feature                      | Spectre  | Kaspa      | Monero  | DERO
-----------------------------|----------|------------|---------|-----------
PoW Algorithm                | SpectreX | kHeavyHash | RandomX | AstroBWTv3
Balance Encryption           | Future   | No         | Yes     | Yes
Transaction Encryption       | Future   | No         | Yes     | Yes
Message Encyrption           | Future   | No         | No      | Yes
Untraceable Transactions     | Future   | No         | Yes     | Yes
Untraceable Mining           | Yes      | No         | No      | Yes
Built-in multicore CPU-miner | Yes      | No         | Yes     | Yes
High BPS                     | Yes      | Yes        | No      | No
High TPS                     | Yes      | Yes        | No      | No

Untraceable Mining is already achieved with AstroBWTv3 and a multicore
miner is already being shipped with Spectre, working on ARM/x86. We
leave it up to the community to build an highly optimized CPU-miner.

## Mathematics

We love numbers, you will find a lot of mathematical constants in the
source code, in the genesis hash, genesis payload, genesis merkle hash
and more. Mathematical constants like [Pi](https://en.wikipedia.org/wiki/Pi),
[E](https://en.wikipedia.org/wiki/E_(mathematical_constant)) and
several prime numbers used as starting values for nonce or difficulty.
The first released version is `0.3.14`, the famous Pi divided by 10.

## Installation

### Binaries

We provide a comprehensive range of pre-compiled binaries for the
Spectre full-node daemon, CLI wallet application, and testing
utilities, all designed to promote decentralization. Here's an
overview of the different builds:

Build                 | Description
----------------------|---------------------------------------------
linux-gnu-aarch64     | Dynamically linked Linux (arm64)
linux-gnu-powerpc64   | Dynamically linked Linux (ppc64)
linux-gnu-powerpc64le | Dynamically linked Linux (ppc64le)
linux-gnu-riscv64     | Dynamically linked Linux (riscv64)
linux-gnu-amd64       | Dynamically linked Linux (x86_64)
linux-musl-aarch64    | Statically linked Linux (arm64)
linux-musl-amd64      | Statically linked Linux (x86_64)
windows-gnullvm-amd64 | Windows version using GNU ABI from Clang/LLVM
windows-msvc-amd64    | Windows version using Microsoft ABI, requires [MSVC runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)
macos-amd64           | macOS version for Intel-based systems
macos-aarch64         | macOS version for Arm-based systems (M1, M2, etc.)

The dynamically linked versions are always preferred for security
reasons. However, for older Linux distributions, statically linked
versions may be necessary due to glibc incompatibilities.

The `windows-msvc-amd64` is recommended for most purposes as it offers
the best interoperability with other Windows software. Note that this
requires accepting the End User License Agreement (EULA) for the
Microsoft Visual C++ Redistributable runtime. You can download and
install the runtime from [here](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170).

Use the `windows-gnullvm-amd64` version if you prefer not to accept
the MSVC runtime EULA.

### Building on Linux

1. Install general prerequisites

   ```bash
   sudo apt install curl git build-essential libssl-dev pkg-config
   ```

2. Install Protobuf (required for gRPC)

   ```bash
   sudo apt install protobuf-compiler libprotobuf-dev #Required for gRPC
   ```

3. Install the clang toolchain (required for RocksDB and WASM secp256k1
   builds)

   ```bash
   sudo apt-get install clang-format clang-tidy \
   clang-tools clang clangd libc++-dev \
   libc++1 libc++abi-dev libc++abi1 \
   libclang-dev libclang1 liblldb-dev \
   libllvm-ocaml-dev libomp-dev libomp5 \
   lld lldb llvm-dev llvm-runtime \
   llvm python3-clang
   ```

3. Install the [rust toolchain](https://rustup.rs/)
   
   If you already have rust installed, update it by running:
   `rustup update`.

4. Install wasm-pack

   ```bash
   cargo install wasm-pack
   ```

4. Install wasm32 target

   ```bash
   rustup target add wasm32-unknown-unknown
   ```      

5. Clone the repo

   ```bash
   git clone https://github.com/spectre-project/rusty-spectre
   cd rusty-spectre
   ```

### Building on Windows

1. [Install Git for Windows](https://gitforwindows.org/) or an alternative Git distribution.

2. Install [Protocol Buffers](https://github.com/protocolbuffers/protobuf/releases/download/v21.10/protoc-21.10-win64.zip) and add the `bin` directory to your `Path`

3. Install [LLVM-15.0.6-win64.exe](https://github.com/llvm/llvm-project/releases/download/llvmorg-15.0.6/LLVM-15.0.6-win64.exe)

   Add the `bin` directory of the LLVM installation
   (`C:\Program Files\LLVM\bin`) to PATH.
    
   Set `LIBCLANG_PATH` environment variable to point to the `bin`
   directory as well.

   **IMPORTANT:** Due to C++ dependency configuration issues, LLVM
   `AR` installation on Windows may not function correctly when
   switching between WASM and native C++ code compilation (native
   `RocksDB+secp256k1` vs WASM32 builds of `secp256k1`). Unfortunately,
   manually setting `AR` environment variable also confuses C++ build
   toolchain (it should not be set for native but should be set for
   WASM32 targets). Currently, the best way to address this, is as
   follows: after installing LLVM on Windows, go to the target `bin`
   installation directory and copy or rename `LLVM_AR.exe` to `AR.exe`.
  
4. Install the [rust toolchain](https://rustup.rs/)
     
   If you already have rust installed, update it by running:
   `rustup update`.

5. Install wasm-pack

   ```bash
   cargo install wasm-pack
   ```

6. Install wasm32 target

   ```bash
   rustup target add wasm32-unknown-unknown
   ```      

7. Clone the repo

   ```bash
   git clone https://github.com/spectre-project/rusty-spectre
   cd rusty-spectre
   ```

### Building on Mac OS

1. Install Protobuf (required for gRPC)

   ```bash
   brew install protobuf
   ```

2. Install LLVM. 
  
   The default XCode installation of `llvm` does not support WASM
   build targets. To build WASM on MacOS you need to install `llvm`
   from homebrew (at the time of writing, the llvm version for MacOS
   is 16.0.1).

   ```bash
   brew install llvm
   ```

   **NOTE:** Homebrew can use different keg installation locations
   depending on your configuration. For example:

   - `/opt/homebrew/opt/llvm` -> `/opt/homebrew/Cellar/llvm/16.0.1`
   - `/usr/local/Cellar/llvm/16.0.1`

   To determine the installation location you can use `brew list llvm`
   command and then modify the paths below accordingly:

   ```bash
   % brew list llvm
   /usr/local/Cellar/llvm/16.0.1/bin/FileCheck
   /usr/local/Cellar/llvm/16.0.1/bin/UnicodeNameMappingGenerator
   ...
   ```

   If you have `/opt/homebrew/Cellar`, then you should be able to use
   `/opt/homebrew/opt/llvm`.

   Add the following to your `~/.zshrc` file:

   ```bash
   export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
   export LDFLAGS="-L/opt/homebrew/opt/llvm/lib"
   export CPPFLAGS="-I/opt/homebrew/opt/llvm/include"
   export AR=/opt/homebrew/opt/llvm/bin/llvm-ar
   ```

   Reload the `~/.zshrc` file:

   ```bash
   source ~/.zshrc
   ```

3. Install the [rust toolchain](https://rustup.rs/)
     
   If you already have rust installed, update it by running:
   `rustup update`.

4. Install wasm-pack

   ```bash
   cargo install wasm-pack
   ```

4. Install wasm32 target

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

5. Clone the repo

   ```bash
   git clone https://github.com/spectre-project/rusty-spectre
   cd rusty-spectre
   ```

### Building WASM32 SDK

Rust WebAssembly (WASM) refers to the use of the Rust programming
language to write code that can be compiled into WebAssembly, a binary
instruction format that runs in web browsers and NodeJs. This allows
for easy development using JavaScript and TypeScript programming
languages while retaining the benefits of Rust.

Spectre on Rust utilizes the SpectreX mining algorithm library for Rust
and leverages `cdivsufsort` for enhanced performance. To compile the
WASM32 SDK using `clang`, additional environment variables need to be
configured:

```
export TARGET_CC=clang
export TARGET_CFLAGS=-I/usr/include
```

WASM SDK components can be built from sources by running:

- `./build-release` - build a full release package (includes both
  release and debug builds for web and nodejs targets)
- `./build-docs` - build TypeScript documentation
- `./build-web` - release web build
- `./build-web-dev` - development web build
- `./build-nodejs` - release nodejs build
- `./build-nodejs-dev` - development nodejs build

**IMPORTANT:** do not use `dev` builds in production. They are
significantly larger, slower and include debug symbols.

#### Requirements

- NodeJs (v20+): https://nodejs.org/en
- TypeDoc: https://typedoc.org/

#### Builds & documentation

- Release builds: https://github.com/spectre-project/rusty-spectre/releases
- Developer TypeScript documentation is available from Kaspa

## Running Spectre CLI + Wallet

`spectre-cli` crate provides cli-driven RPC interface to the node and
a terminal interface to the Rusty Spectre Wallet runtime. These wallets
are compatible with WASM SDK Wallet API and Spectre NG projects.

```bash
cd cli
cargo run --release
```

## Running Local Web Wallet

Run an http server inside of `wallet/wasm/web` folder. If you don't
have once, you can use the following:

```bash
cd wallet/wasm/web
cargo install basic-http-server
basic-http-server
```

The *basic-http-server* will serve on port 4000 by default, so open
your web browser and load http://localhost:4000

The framework is compatible with all major desktop and mobile browsers.

## Running the node

Start a mainnet node:

```bash
cargo run --release --bin spectred
```

Start a testnet node:

```bash
cargo run --release --bin spectred -- --testnet
```

Using a configuration file

```bash
cargo run --release --bin spectred -- --configfile /path/to/configfile.toml
# or
cargo run --release --bin spectred -- -C /path/to/configfile.toml
```

- The config file should be a list of \<CLI argument\> = \<value\>
  separated by newlines. 
- Whitespace around the `=` is fine, `arg=value` and `arg = value`
  are both parsed correctly.
- Values with special characters like `.` or `=` will require quoting
  the value i.e \<CLI argument\> = "\<value\>".
- Arguments with multiple values should be surrounded with brackets
  like `addpeer = ["10.0.0.1", "1.2.3.4"]`.

For example:
```
testnet = true
utxoindex = false
disable-upnp = true
perf-metrics = true
appdir = "some-dir"
netsuffix = 11
addpeer = ["10.0.0.1", "1.2.3.4"]
```

Pass the `--help` flag to view all possible arguments.

```bash
cargo run --release --bin spectred -- --help
```

## wRPC

wRPC subsystem is disabled by default in `spectred` and can be enabled via:

JSON protocol:

```bash
--rpclisten-json = <interface:port>
```

Borsh protocol:

```bash
--rpclisten-borsh = <interface:port>
```

### Sidenote

Rusty Spectre integrates an optional wRPC subsystem. wRPC is a
high-performance, platform-neutral, Rust-centric, WebSocket-framed
RPC implementation that can use [Borsh](https://borsh.io/) and JSON
protocol encoding.

JSON protocol messaging is similar to JSON-RPC 1.0, but differs from
the specification due to server-side notifications.

[Borsh](https://borsh.io/) encoding is meant for inter-process
communication. When using [Borsh](https://borsh.io/) both client and
server should be built from the same codebase.

JSON protocol is based on Spectre data structures and is
data-structure-version agnostic. You can connect to the JSON endpoint
using any WebSocket library. Built-in RPC clients for JavaScript and
TypeScript capable of running in web browsers and Node.js are
available as a part of the Spectre WASM framework.

**wRPC to gRPC Proxy is deprecated and no longer supported.**

## Mining

Mining is currently supported on all networks, so once you've setup a
node, follow these instructions. Download and unzip the latest miner
binaries from [spectre-project/spectre-miner](https://github.com/spectre-project/spectre-miner/releases).
In a separate terminal run the miner:

```
./spectre-miner --mining-address spectre:qrxf48dgrdkjxllxczek3uweuldtan9nanzjsavk0ak9ynwn0zsayjjh7upez
```

You can replace the above mining address with your own address by
creating one.
