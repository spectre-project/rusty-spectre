# Spectre Testnet 8 (TN8) – Sigma Hardfork Node Setup Guide

Spectre is about to take a significant leap with the **Sigma Hardfork**, following a similar approach to [KIP14](https://github.com/kaspanet/kips/blob/master/kip-0014.md), but transitioning from 1 to 8 blocks per second instead of 10 BPS. By running TN8 and providing feedback, you help prepare for a smooth mainnet upgrade, planned for May.

---

## Recommended Hardware Specifications

- **Minimum**:

  - 8 CPU cores
  - 16 GB RAM
  - 256 GB SSD
  - 5 MB/s (or ~40 Mbit/s) network bandwidth

- **Preferred for Higher Performance**:
  - 12–16 CPU cores
  - 32 GB RAM
  - 512 GB SSD
  - Higher network bandwidth for robust peer support

While the minimum specs suffice to sync and maintain a TN8 node with the accelerated 8 bps, increasing CPU cores, RAM, storage, and bandwidth allows your node to serve as a stronger focal point on the network. This leads to faster initial block download (IBD) for peers syncing from your node and provides more leeway for future storage growth and optimization.

---

## 1. Install & Run Your TN8 Node

1. **Obtain Spectre binaries**  
   Download and extract the official [latest-release](https://github.com/spectre-project/rusty-spectre/releases/latest), or build from the `main` branch by following the instructions in the project README.

2. **Launch the Node**  
   While TN8 is the default netsuffix, specifying it explicitly is recommended:

   ```
   spectred --testnet --netsuffix=8 --utxoindex
   ```

   _(If running from source code:)_

   ```
   cargo run --bin spectred --release -- --testnet --netsuffix=8 --utxoindex
   ```

Leave this process running. Closing it will stop your node.

- **Advanced Command-Line Options**:
  - `--rpclisten=0.0.0.0` to listen for RPC connections on all network interfaces (public RPC).
  - `--rpclisten-borsh` for local borsh RPC access from the `spectre-cli` binary.
  - `--unsaferpc` for allowing P2P peer query and management via RPC (recommended to use only if **not** exposing RPC publicly).
  - `--perf-metrics --loglevel=info,spectred_lib::daemon=debug,spectre_mining::monitor=debug` for detailed performance logs.
  - `--loglevel=spectre_grpc_server=warn` for suppressing most RPC connect/disconnect log reports.
  - `--ram-scale=3.0` for increasing cache size threefold (relevant for utilizing large RAM; can be set between 0.1 and 10).

---

## 2. Generate Transactions with Rothschild

1. **Create a Wallet**

```
rothschild
```

This outputs a private key and a public address. Fund your wallet by mining to it or obtaining test coins from other TN8 participants.

2. **Broadcast Transactions**

```
rothschild --private-key <your-private-key> -t=10
```

Replace <your-private-key> with the key from step 1. The `-t=10` flag sets your transaction rate to 10 TPS (feel free to try different rates, but keep it below 50 TPS).

---

## 3. Mining on TN8

1. **Download the Miner**  
   Use the latest Spectre CPU miner [latest-release](https://github.com/spectre-project/spectre-miner/releases/latest) which supports TN8.

2. **Start Mining**

```
spectre-miner --testnet --mining-address <your-address> -p 18210 -t 1
```

Replace <your-address> with your TN8 address (e.g., from Rothschild) if you want to mine and generate transactions simultaneously.

---

## Summary & Next Steps

- **Node Sync:**  
  `spectred --testnet --netsuffix=8 --utxoindex`
- **Transaction Generation:**  
  `rothschild --private-key <your-private-key> -t=10`
- **Mining:**  
  `spectre-miner --testnet --mining-address <your-address> -p 18210 -t 1`
