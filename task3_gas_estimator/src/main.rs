use ethers::providers::{Http, Middleware, Provider};
use ethers::types::U256;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rpc_url =
        std::env::var("ARBITRUM_RPC_URL").unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    let gas_limit: u64 = 21_000;
    let (gas_price_wei, gas_fee_wei, gas_fee_eth) = estimate_transfer_gas_fee(&rpc_url, gas_limit).await?;

    println!("当前 Gas 价格: {} wei", gas_price_wei);
    println!("基础转账 Gas 限额: {}", gas_limit);
    println!("预估转账 Gas 费: {} wei (~{} ETH)", gas_fee_wei, gas_fee_eth);

    Ok(())
}

async fn estimate_transfer_gas_fee(
    rpc_url: &str,
    gas_limit: u64,
) -> Result<(U256, U256, f64), Box<dyn Error>> {
    let provider = Provider::<Http>::try_from(rpc_url)?;

    let gas_price_wei: U256 = provider.get_gas_price().await?;

    let gas_limit_u256 = U256::from(gas_limit);
    let gas_fee_wei = gas_price_wei * gas_limit_u256;

    let gas_fee_wei_str = gas_fee_wei.to_string();
    let gas_fee_wei_f64: f64 = gas_fee_wei_str.parse()?;
    let gas_fee_eth = gas_fee_wei_f64 / 1e18_f64;

    Ok((gas_price_wei, gas_fee_wei, gas_fee_eth))
}
