# 套利利润保护合约

这是一个只用来检查是否存在套利利润的合约程序，而非是一个独立的套利机器人合约，通常合约需要与链下程序一起使用。且只有一个核心功能，就是判断套利交易是否存在利润。

假如当使用 [jup](https://jup.ag/) 的 [/quote](https://dev.jup.ag/docs/api/swap-api/quote) 和 [/swap-instructions](https://dev.jup.ag/docs/api/swap-api/swap-instructions) 接口进行套利程序开发时，由于从发现利润到上链都需要时间，经常出现上链后，由于交易价格快速发生变化，导致本来的利润变成了亏损。特别是在网络延时比较严重的时候，这种情况更是一种常见现象，因此需要对利润进行保护，这正是当前合约要发挥的作用。

而当接入本合约后，执行交易时如果合约程序发现存在利润小于用户指定的最小利润 `min_profit` ，则进行整个交易回滚减少套利损失。否则正常完执行交易。

为了减少合约大小，本合约采用 `Rust Native` 开发，并未采用 `Anchor` 开发框架。因为是因为使用 Anchor 开发同样的一个小功能，部署时需要花费的费用是 Rust Native 的 8.5 倍之多。



## 调用代码

```rust
// 接收小费账号，目前合约源码未启用此功能，如有需要自行注释掉相关代码即可
let fee_recipient_pubkey = FEE_RECIPIENT_PUBKEY;

// 最小利润，低于此利润，交易将回滚，防止套利失败造成的重大损失
let min_profit: u64 = 50000;

// 开始套利前，账户余额（单位 Lamport), 如这里表示 3 Sol
let current_balance: u64 = 3_000_000_000


// 指令数据
let mut instruction_data = Vec::new();
instruction_data.extend_from_slice(&min_profit.to_le_bytes());
instruction_data.extend_from_slice(&current_balance.to_le_bytes());

Instruction {
    program_id: CHECK_PROFIT_PROGRAM_ID,
    accounts: vec![
        AccountMeta::new(payer_pubkey, true), // Payer
        AccountMeta::new(fee_recipient_pubkey, false),
        AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false), // Check Profit Account
    ],
    data: instruction_data,
}
```

`Instruction` 相关的三个主要参数说明：

- program_id 参数

	-  `CHECK_PROFIT_PROGRAM_ID` 这个是部署的合约ID

- `accounts` 参数
	 - `payer_pubkey` 扣除小费账号，账户需要签名状态
	 - `fee_recipient_pubkey`  接受利润小费账号，账户需要可修改状态
	 - `solana_sdk_ids::system_program::ID`这个是固定的值 `11111111111111111111111111111111`
	
- `data` 参数

   这里表示要传递的指令参数，一共两个参数，分别为 `min_profit` 和 `current_balance` ，两者均为 `u64` 数据类型


> [!NOTE]
> 
>合约部署者如果想通过合约收取小费是通过 `fee_recipient_pubkey` 来接收的(合约里需要开发者自行添加对小费账户的合法性检查逻辑)，只在有利润的情况下才会收费。目前功能未启用，如需要注释掉合约代码即可

## 交易日志

交易成功与失败的日志输出如下

### 成功交易

```coffeescript
#10 Unknown Program (2Ub8nv4khFWaSwJCDda7UMFxviE6Td99VSyLdhiNTNKJ) instruction
> Program log: 732032473 - 732017034 = 15439 [8000]
> Program 2Ub8nv4khFWaSwJCDda7UMFxviE6Td99VSyLdhiNTNKJ consumed 2313 of 973023 compute units
> Program returned success
```



失败交易

```coffeescript
#10 Unknown Program (2Ub8nv4khFWaSwJCDda7UMFxviE6Td99VSyLdhiNTNKJ) instruction
> Program log: 731452255 - 732032034 = -579779 [8000]
> Program log: No profitable arbitrage found
> Program 2Ub8nv4khFWaSwJCDda7UMFxviE6Td99VSyLdhiNTNKJ consumed 2697 of 1174795 compute units
> Program returned error: custom program error: 0x64
```

失败时打印 `custom program error: 0x64` 错误码

## 编译部署

环境

```shell
$ solana config get
Config File: /Users/Marks/.config/solana/cli/config.yml
RPC URL: https://api.mainnet-beta.solana.com
WebSocket URL: wss://api.mainnet-beta.solana.com/ (computed)
Keypair Path: /Users/Marks/.config/solana/id.json
Commitment: confirmed
```

这里是部署到 solana 主网，如果与您的不一样，可有需要根据需求自行切换到想要的网络。

编译

```shell
$ cargo build-sbf
```

会在项目根目录自动生成一个 `./target/deploy`文件夹，里面包含有编译后的 `.so` 文件 和 当前合约账户的 `json`文件。

部署

```shell
$ solana program deploy ./target/deploy/hello_world.so 
```

> 这里的文件名 hello_world 是在 Cargo.toml 文件里的 name 字段定义的，可自行修改。

这将直接部署到solana链上，同时花费一定的sol费用。

> [!IMPORTANT]
>
> 合约越小越好，当前合约未启用接收小费功能，大小为 `23k` ，花费大概 `0.2 sol`。而如果启用接收小费功能，则变大为 `72K` 大小，这时部署这个合约可能要花费大约 `0.5 sol`，请在部署合约前确保账户余额足够多。
>
> 而如何采用 `Anchor` 开发的话，则大概需要 `1.2 sol `， 开发者可能需要注意到这一点。
