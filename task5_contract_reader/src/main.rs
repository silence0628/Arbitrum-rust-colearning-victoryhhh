use ethers::abi::Abi;
use ethers::contract::{abigen, Contract};
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use serde_json;
use std::env;
use std::error::Error;
use std::fs;
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

    let weth = Weth9::new(contract_address, provider.clone().into());

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

    let abi_path = env::var("WETH_ABI_PATH")
        .unwrap_or_else(|_| "abis/weth9_minimal.json".to_string());
    let abi_json = fs::read_to_string(&abi_path)?;
    let abi: Abi = serde_json::from_str(&abi_json)?;

    let contract = Contract::new(contract_address, abi, provider.clone().into());
    let name_from_json: String = contract
        .method::<_, String>("name", ())?
        .call()
        .await?;
    println!("Token name (from JSON ABI): {}", name_from_json);

    if let Ok(query_address_str) = env::var("QUERY_ADDRESS") {
        if let Ok(query_address) = Address::from_str(&query_address_str) {
            let balance: U256 = weth.balance_of(query_address).call().await?;
            println!("Balance of {}: {}", query_address, balance);
        }
    }

    Ok(())
}
