use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use std::env;
use std::error::Error;
use std::str::FromStr;

abigen!(
    Weth9,
    r#"[
        function name() view returns (string)
        function symbol() view returns (string)
        function decimals() view returns (uint8)
        function totalSupply() view returns (uint256)
        function balanceOf(address owner) view returns (uint256)
    ]"#
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rpc_url = env::var("ARBITRUM_RPC_URL")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    let chain_id: u64 = env::var("ARBITRUM_CHAIN_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(421_614);

    let contract_address_str = env::var("CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "0x2836ae2ea2c013acd38028fd0c77b92cccfa2ee4".to_string());

    let contract_address = Address::from_str(&contract_address_str)?;

    let provider = Provider::<Http>::try_from(rpc_url.clone())?;

    let weth = Weth9::new(contract_address, provider.into());

    let name: String = weth.name().call().await?;
    let symbol: String = weth.symbol().call().await?;
    let decimals: u8 = weth.decimals().call().await?;
    let total_supply: U256 = weth.total_supply().call().await?;

    println!("RPC URL: {}", rpc_url);
    println!("Chain ID: {}", chain_id);
    println!("Contract address: {:?}", contract_address);
    println!("Token name: {}", name);
    println!("Token symbol: {}", symbol);
    println!("Token decimals: {}", decimals);
    println!("Total supply (raw): {}", total_supply);

    if let Ok(query_address_str) = env::var("QUERY_ADDRESS") {
        if let Ok(query_address) = Address::from_str(&query_address_str) {
            let balance: U256 = weth.balance_of(query_address).call().await?;
            println!("Balance of {}: {}", query_address, balance);
        }
    }

    Ok(())
}
