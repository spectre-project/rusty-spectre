// Example of simple client to connect with Spectre node using wRPC connection and collect some node and network basic data

use spectre_rpc_core::{api::rpc::RpcApi, GetBlockDagInfoResponse, GetServerInfoResponse};
use spectre_wrpc_client::{
    client::{ConnectOptions, ConnectStrategy},
    prelude::NetworkId,
    prelude::NetworkType,
    result::Result,
    Resolver, SpectreRpcClient, WrpcEncoding,
};
use std::process::ExitCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> ExitCode {
    match check_node_status().await {
        Ok(_) => {
            println!("Well done! You successfully completed your first client connection to Spectre node!");
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!("An error occurred: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn check_node_status() -> Result<()> {
    // Select encoding method to use, depending on node settings
    let encoding = WrpcEncoding::Borsh;

    // If you want to connect to your own node, define your node address and wRPC port using let url = Some("ws://0.0.0.0:17110")
    // Verify your Spectre node is runnning with --rpclisten-borsh=0.0.0.0:17110 parameter
    // In this example we don't use a specific node but we connect through the resolver, which use a pool of public nodes
    let url = None;
    let resolver = Some(Resolver::default());

    // Define the network your Spectre node is connected to
    // You can select NetworkType::Mainnet, NetworkType::Testnet, NetworkType::Devnet, NetworkType::Simnet
    let network_type = NetworkType::Mainnet;
    let selected_network = Some(NetworkId::new(network_type));

    // Advanced options
    let subscription_context = None;

    // Create new wRPC client with parameters defined above
    let client = SpectreRpcClient::new(encoding, url, resolver, selected_network, subscription_context)?;

    // Advanced connection options
    let timeout = 5_000;
    let options = ConnectOptions {
        block_async_connect: true,
        connect_timeout: Some(Duration::from_millis(timeout)),
        strategy: ConnectStrategy::Fallback,
        ..Default::default()
    };

    // Connect to selected Spectre node
    client.connect(Some(options)).await?;

    // Retrieve and show Spectre node information
    let GetServerInfoResponse { is_synced, server_version, network_id, has_utxo_index, .. } = client.get_server_info().await?;

    println!("Node version: {server_version}");
    println!("Network: {network_id}");
    println!("Node is synced: {is_synced}");
    println!("Node is indexing UTXOs: {has_utxo_index}");

    // Retrieve and show Spectre network information
    let GetBlockDagInfoResponse {
        block_count,
        header_count,
        tip_hashes,
        difficulty,
        past_median_time,
        virtual_parent_hashes,
        pruning_point_hash,
        virtual_daa_score,
        sink,
        ..
    } = client.get_block_dag_info().await?;

    println!("Block count: {block_count}");
    println!("Header count: {header_count}");
    println!("Tip hashes:");
    for tip_hash in tip_hashes {
        println!("{tip_hash}");
    }
    println!("Difficulty: {difficulty}");
    println!("Past median time: {past_median_time}");
    println!("Virtual parent hashes:");
    for virtual_parent_hash in virtual_parent_hashes {
        println!("{virtual_parent_hash}");
    }
    println!("Pruning point hash: {pruning_point_hash}");
    println!("Virtual DAA score: {virtual_daa_score}");
    println!("Sink: {sink}");

    // Disconnect client from Spectre node
    client.disconnect().await?;

    // Return function result
    Ok(())
}
