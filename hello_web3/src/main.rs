//! Example of creating an HTTP provider using the `connect_http` method on the `ProviderBuilder`.
 
use alloy::providers::ProviderBuilder; 
use std::error::Error;
 
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up the HTTP transport which is consumed by the RPC client.
    let rpc_url = "https://reth-ethereum.ithaca.xyz/rpc".parse()?;

    // Create a provider with the HTTP transport using the `reqwest` crate.
    let _provider = ProviderBuilder::new().connect_http(rpc_url); 
    println!("连接成功");

    Ok(())
}
