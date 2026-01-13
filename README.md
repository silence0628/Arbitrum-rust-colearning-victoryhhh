Arbitrum Rust 共学项目总览
==========================

本仓库包含多个独立的 Rust 小任务，通过逐步实践熟悉 Arbitrum 测试网的基础交互能力：

- 连接链节点
- 查询账户余额
- 估算 Gas 费用
- 构造并发送 EIP-1559 转账
- 调用链上已部署合约的只读方法

---

Task 1：hello_web3
------------------

- 目标：用 Rust 连接 Arbitrum 测试网，完成最基础的 RPC 通信验证。
- 目录：`hello_web3/`
- 说明：
  - 使用 `tokio` 异步运行时。
  - 使用 alloy 相关依赖与节点建立连接，并打印简单信息（例如连接成功提示）。

运行示例：

```bash
cd hello_web3
cargo run
```

---

Task 2：task2_balance_checker（查询地址 ETH 余额）
-----------------------------------------------

- 目标：使用 `ethers-rs` 查询 Arbitrum 测试网上指定地址的 ETH 余额，并从 wei 转换为可读的 ETH。
- 目录：`task2_balance_checker/`
- 要点：
  - 通过 HTTP RPC 连接 Arbitrum 测试网。
  - 支持从字符串解析以太坊地址。
  - 使用 `ethers::providers` 查询余额，结果从 `wei` 转换为 `ETH` 后输出。

运行示例（示意）：

```bash
cd task2_balance_checker
cargo run
```

---

Task 3：task3_gas_estimator（估算基础转账 Gas 费用）
----------------------------------------------

- 目标：用 `ethers-rs` 动态获取 Arbitrum 测试网当前 Gas 价格，并结合基础转账 Gas 限额估算转账手续费。
- 目录：`task3_gas_estimator/`
- 要点：
  - 动态获取链上实时 Gas 价格，而不是写死固定值。
  - 使用行业通用的基础转账 Gas 限额（或估算值）。
  - 根据公式：`Gas 费 = Gas 价格 × Gas 限额` 计算并打印出预估手续费。

运行示例：

```bash
cd task3_gas_estimator
cargo run
```

---

Task 4：task4_transfer_script（EIP-1559 转账脚本）
--------------------------------------------

- 目标：实现一个安全、可配置的 Arbitrum 测试网转账脚本，支持 EIP-1559 手续费、动态 Gas 限额估算与私钥环境变量管理。
- 目录：`task4_transfer_script/`
- 核心特性：
  - **安全性**：不在代码中硬编码私钥，只从环境变量读取：
    - `SENDER_PRIVATE_KEY`：发送方私钥（16 进制，不含 `0x`）
    - `RECIPIENT_ADDRESS`：接收方地址
  - **网络配置**：
    - `ARBITRUM_RPC_URL`：Arbitrum RPC（有默认值：`https://sepolia-rollup.arbitrum.io/rpc`）
    - `ARBITRUM_CHAIN_ID`：链 ID（默认 `421614`，Arbitrum Sepolia）
  - **手续费与 Gas**：
    - 使用 `estimate_gas` 动态估算本次转账所需 `gas_limit`，避免 “intrinsic gas too low”。
    - 根据最新区块 `base_fee_per_gas` 计算：
      - `max_priority_fee_per_gas` 固定为 1 gwei。
      - `max_fee_per_gas = base_fee * 2 + priority_fee`，从职业角度给出安全裕量，避免 “max fee per gas less than block base fee”。
    - 打印本次转账的预估 Gas 费上限（wei 和 ETH）。
  - **交易结果**：
    - 构造 EIP-1559 交易并发送，打印 `tx hash`。
    - 等待上链并输出完整 `TransactionReceipt`。

运行示例：

```bash
cd task4_transfer_script

export ARBITRUM_RPC_URL="https://sepolia-rollup.arbitrum.io/rpc"
export ARBITRUM_CHAIN_ID=421614
export SENDER_PRIVATE_KEY="你的私钥（不带0x）"
export RECIPIENT_ADDRESS="0x接收地址"
export TRANSFER_AMOUNT_ETH="0.000001"  # 可选，默认 0.000001

cargo run
```

---

Task 5：task5_contract_reader（调用链上 WETH9 合约）
----------------------------------------------

- 目标：在 **不部署新合约** 的前提下，使用 `ethers-rs` 调用 Arbitrum 测试网中已部署的 WETH9 合约，只读查询合约信息。
- 目录：`task5_contract_reader/`
- 使用合约：
  - 网络：Arbitrum Sepolia
  - 合约：`WETH9`（Wrapped Ether）
  - 默认合约地址：`0x2836ae2ea2c013acd38028fd0c77b92cccfa2ee4`
- 技术要点：
  - 使用 `abigen!` 宏，根据最小 ABI 生成强类型合约绑定：
    - `name() -> String`
    - `symbol() -> String`
    - `decimals() -> u8`
    - `totalSupply() -> U256`
    - `balanceOf(address) -> U256`
  - 只需要 `Provider<Http>` 即可完成只读调用，无需签名。
  - 支持通过环境变量配置合约地址与查询地址：
    - `ARBITRUM_RPC_URL`：RPC（默认 `https://sepolia-rollup.arbitrum.io/rpc`）
    - `ARBITRUM_CHAIN_ID`：链 ID（默认 `421614`）
    - `CONTRACT_ADDRESS`：可选，覆盖默认 WETH9 地址
    - `QUERY_ADDRESS`：可选，用于调用 `balanceOf(QUERY_ADDRESS)`。

运行示例：

```bash
cd task5_contract_reader

export ARBITRUM_RPC_URL="https://sepolia-rollup.arbitrum.io/rpc"
export ARBITRUM_CHAIN_ID=421614
# 可选：覆盖默认合约地址
# export CONTRACT_ADDRESS="0x2836ae2ea2c013acd38028fd0c77b92cccfa2ee4"

# 可选：查询某个地址的 WETH 余额
# export QUERY_ADDRESS="0x你的测试地址"

cargo run
```

---

后续可以在此总文档中继续补充更多 Task 说明（如测试命令、截图链接等），用于快速回顾每个练习的目标与入口。
