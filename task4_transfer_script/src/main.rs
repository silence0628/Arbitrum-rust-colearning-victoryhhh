use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{
    transaction::eip1559::Eip1559TransactionRequest,
    transaction::eip2718::TypedTransaction,
    Address, BlockId, BlockNumber, TransactionRequest, U256,
};
use ethers::middleware::SignerMiddleware;
use ethers::utils::parse_ether;
use std::env;
use std::error::Error;
use std::str::FromStr;

// 程序总功能：
// 从环境变量读取 Arbitrum 测试网配置，估算 EIP-1559 转账所需 Gas 与手续费，
// 并构造、发送一笔实际转账交易，打印交易哈希和上链回执。

type EthProvider = Provider<Http>;
type EthClient = SignerMiddleware<EthProvider, LocalWallet>;

// 运行时配置：包含 RPC、链 ID、私钥、收款地址和转账金额
struct TransferConfig {
    rpc_url: String,
    chain_id: u64,
    private_key: String,
    recipient: Address,
    amount_eth_str: String,
}

// 手续费相关信息：基础费、小费、上限、Gas 限额和预估总费用
struct GasFeeInfo {
    base_fee_per_gas: U256,
    max_priority_fee_per_gas: U256,
    max_fee_per_gas: U256,
    gas_limit: U256,
    gas_fee_wei: U256,
    gas_fee_eth: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. 从环境变量加载转账配置，如果缺少必填项则直接提示并退出
    let config = match load_config_from_env() {
        Ok(cfg) => cfg,
        Err(msg) => {
            println!("{}", msg);
            return Ok(());
        }
    };

    // 2. 基于 RPC URL 创建 Provider，用于与 Arbitrum 节点交互
    let provider = create_provider(&config.rpc_url)?;

    // 3. 基于私钥和链 ID 创建钱包，并绑定链 ID
    let wallet = create_wallet(&config.private_key, config.chain_id)?;
    let from = wallet.address(); // 获取发送方地址

    println!("发送方地址: {:?}", from);
    println!("接收方地址: {:?}", config.recipient);
    println!("使用的 RPC: {}", config.rpc_url);
    println!("链 ID: {}", config.chain_id);

    // 4. 将转账金额从字符串（ETH）转换为 U256（wei）
    let amount_wei: U256 = parse_ether(&config.amount_eth_str)?;

    // 5. 构造用于估算 Gas 的 Legacy 交易，并向节点请求 Gas 限额
    let tx_for_estimate =
        build_legacy_tx_for_estimate(from, config.recipient, amount_wei);
    let gas_limit = estimate_gas_limit(&provider, &tx_for_estimate).await?;

    // 6. 基于最新区块 Base Fee 计算 EIP-1559 手续费参数并打印预估
    let gas_fee_info = calculate_gas_fee_info(&provider, gas_limit).await?;

    // 7. 创建带签名能力的中间件客户端，用于发送交易
    let client = EthClient::new(provider, wallet);

    // 8. 构造并发送实际的 EIP-1559 转账交易，等待上链回执
    send_eip1559_transfer(
        &client,
        from,
        config.recipient,
        amount_wei,
        &gas_fee_info,
    )
    .await?;

    Ok(())
}

// 从环境变量读取并校验转账相关配置
fn load_config_from_env() -> Result<TransferConfig, String> {
    // RPC 地址，缺省时使用 Arbitrum Sepolia 公共 RPC
    let rpc_url = env::var("ARBITRUM_RPC_URL")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    // 发送方私钥，必须存在且为 16 进制字符串（不含 0x）
    let private_key = match env::var("SENDER_PRIVATE_KEY") {
        Ok(v) => v,
        Err(_) => {
            return Err(
                "缺少环境变量 SENDER_PRIVATE_KEY，请设置发送方私钥（16进制，不含0x）"
                    .to_string(),
            )
        }
    };

    // 接收方地址，必须存在且为合法的以太坊地址
    let to_address_str = match env::var("RECIPIENT_ADDRESS") {
        Ok(v) => v,
        Err(_) => {
            return Err(
                "缺少环境变量 RECIPIENT_ADDRESS，请设置接收方 Arbitrum 地址".to_string(),
            )
        }
    };

    // 转账金额（单位 ETH），默认 0.000001 ETH
    let amount_eth_str =
        env::var("TRANSFER_AMOUNT_ETH").unwrap_or_else(|_| "0.000001".to_string());

    // 链 ID，默认使用 Arbitrum Sepolia（421614）
    let chain_id: u64 = env::var("ARBITRUM_CHAIN_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(421_614);

    // 将收款地址从字符串解析为 Address 类型
    let recipient: Address = match Address::from_str(&to_address_str) {
        Ok(addr) => addr,
        Err(_) => {
            return Err(
                "RECIPIENT_ADDRESS 非法，请确认是有效的 Arbitrum 地址".to_string(),
            )
        }
    };

    Ok(TransferConfig {
        rpc_url,
        chain_id,
        private_key,
        recipient,
        amount_eth_str,
    })
}

// 基于 RPC URL 创建 HTTP Provider
fn create_provider(rpc_url: &str) -> Result<EthProvider, Box<dyn Error>> {
    // try_from 会校验 URL 格式并创建底层 HTTP 客户端
    let provider = EthProvider::try_from(rpc_url.to_string())?;
    Ok(provider)
}

// 基于私钥和链 ID 创建本地钱包
fn create_wallet(
    private_key: &str,
    chain_id: u64,
) -> Result<LocalWallet, Box<dyn Error>> {
    // 从 16 进制私钥字符串解析出 LocalWallet
    let wallet: LocalWallet = private_key.parse()?;
    // 绑定链 ID，确保签名的交易属于正确网络
    Ok(wallet.with_chain_id(chain_id))
}

// 构造用于 Gas 估算的 Legacy 交易，再包装为 EIP-2718 TypedTransaction
fn build_legacy_tx_for_estimate(
    from: Address,
    to: Address,
    amount_wei: U256,
) -> TypedTransaction {
    // 只需设置 from / to / value 即可用于估算基础转账的 Gas
    let tx = TransactionRequest::new()
        .to(to)
        .from(from)
        .value(amount_wei);

    // estimate_gas 接受 TypedTransaction，这里使用 Legacy 形式
    TypedTransaction::Legacy(tx)
}

// 调用节点的 estimate_gas 接口，估算本次转账所需 Gas 限额
async fn estimate_gas_limit(
    provider: &EthProvider,
    tx: &TypedTransaction,
) -> Result<U256, Box<dyn Error>> {
    // 将未签名交易发送给节点，让节点模拟执行并返回所需的 Gas
    let estimated_gas: U256 = provider.estimate_gas(tx, None).await?;
    Ok(estimated_gas)
}

// 查询最新区块的 Base Fee，并结合 Gas 限额计算 EIP-1559 手续费参数
async fn calculate_gas_fee_info(
    provider: &EthProvider,
    gas_limit: U256,
) -> Result<GasFeeInfo, Box<dyn Error>> {
    // 获取最新区块，用于读取当前 base_fee_per_gas
    let latest_block = provider
        .get_block(BlockId::Number(BlockNumber::Latest))
        .await?;

    // 如果区块中没有 base_fee_per_gas 字段，则回退为 0
    let base_fee_per_gas = latest_block
        .and_then(|b| b.base_fee_per_gas)
        .unwrap_or_else(|| U256::from(0u64));

    // 给矿工的小费，固定设置为 1 gwei
    let max_priority_fee_per_gas: U256 = U256::from(1_000_000_000u64);

    // 为了避免 max_fee < base_fee + priority_fee，使用 2 倍 base_fee 作为缓冲
    let max_fee_per_gas: U256 = base_fee_per_gas * 2 + max_priority_fee_per_gas;

    // 计算在最坏情况下（按 max_fee_per_gas 全额计价）的总 Gas 费用
    let gas_fee_wei = max_fee_per_gas * gas_limit;
    let gas_fee_eth = gas_fee_wei.to_string().parse::<f64>()? / 1e18_f64;

    // 打印当前 Base Fee 和本次交易的预估 Gas 成本信息
    println!("当前 base_fee_per_gas: {} wei", base_fee_per_gas);
    println!(
        "使用的 max_fee_per_gas: {} wei, max_priority_fee_per_gas: {} wei",
        max_fee_per_gas, max_priority_fee_per_gas
    );
    println!("基础转账 Gas 限额(估算): {}", gas_limit);
    println!(
        "预估转账 Gas 费上限: {} wei (~{} ETH)",
        gas_fee_wei, gas_fee_eth
    );

    Ok(GasFeeInfo {
        base_fee_per_gas,
        max_priority_fee_per_gas,
        max_fee_per_gas,
        gas_limit,
        gas_fee_wei,
        gas_fee_eth,
    })
}

// 使用 EIP-1559 交易格式发送实际转账，并打印交易哈希与上链回执
async fn send_eip1559_transfer(
    client: &EthClient,
    from: Address,
    to: Address,
    amount_wei: U256,
    gas_fee_info: &GasFeeInfo,
) -> Result<(), Box<dyn Error>> {
    // 构造 EIP-1559 转账交易，带上 Gas 限额与手续费参数
    let tx = Eip1559TransactionRequest::new()
        .to(to)
        .from(from)
        .value(amount_wei)
        .gas(gas_fee_info.gas_limit)
        .max_fee_per_gas(gas_fee_info.max_fee_per_gas)
        .max_priority_fee_per_gas(gas_fee_info.max_priority_fee_per_gas);

    // 发送交易，返回 PendingTransaction 句柄
    let pending_tx = client.send_transaction(tx, None).await?;
    let tx_hash = *pending_tx; // 直接从 PendingTransaction 中解构出交易哈希
    println!("交易已发送，tx hash: {:?}", tx_hash);

    // 异步等待交易被打包上链，并打印回执
    let receipt = pending_tx.await?;
    println!("交易已上链，receipt: {:?}", receipt);

    Ok(())
}
