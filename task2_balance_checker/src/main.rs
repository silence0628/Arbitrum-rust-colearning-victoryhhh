use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, U256};
use std::error::Error;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // ① Arbitrum 测试网 RPC（这里默认用 Arbitrum Sepolia）
    let rpc_url =
        std::env::var("ARBITRUM_RPC_URL").unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    // ② 示例地址：可以替换成你自己的 Arbitrum 测试网地址
    let addr_str = "0x34e6E1814300DF9237F52B2c11509FA9fb280FE3";

    let balance_eth = get_eth_balance_on_arbitrum(&rpc_url, addr_str).await?;
    println!("地址 {addr_str} 在 Arbitrum 测试网的余额：{balance_eth} ETH");

    Ok(())
}

/// 查询指定地址在 Arbitrum 测试网的 ETH 余额，并转换为 ETH 单位
async fn get_eth_balance_on_arbitrum(
    rpc_url: &str,
    address: &str,
) -> Result<f64, Box<dyn Error>> {
    // 创建 HTTP Provider
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // 解析地址字符串
    let addr = Address::from_str(address)?;

    // 查询最新区块的余额（单位：wei）
    let balance_wei: U256 = provider.get_balance(addr, None).await?;

    // 将 U256 转成 f64（只用于展示，存在精度损失，但足够演示）
    let wei_as_f64 = balance_wei.to_string().parse::<f64>()?;
    let eth = wei_as_f64 / 1e18_f64;

    Ok(eth)
}
