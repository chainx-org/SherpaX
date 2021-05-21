# Uniswap接口文档

## RPC

#### getAmountInPrice

---

###### 接口功能
> 查询买入汇率

###### 请求参数
> | 参数       | 必选 | 类型         | 说明                                 |
> | :--------- | :--- | :----------- | ------------------------------------ |
> | amount_out | ture | u128         | 输出量（swap操作右边固定，查询左边） |
> | path       | true | Vec<AssetId> | 兑换路径                             |

###### 返回字段
> | 返回字段 | 字段类型 | 说明             |
> | :------- | :------- | :--------------- |
> | 数值     | Hex      | 计算得到的输入量 |

###### http请求示例
> ~~~
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>         "jsonrpc": "2.0",
>         "id": 1059,
>         "method": "swap_getAmountInPrice",
>         "params": [100,[0, 1]]
>   }'
> ~~~
###### Response

> ```json
> {
>     "jsonrpc": "2.0",
>     "result": "0x64",
>     "id": 1059
> }
> ```

###### ws请求示例

> ```
> api.rpc.swap.getAmountInPrice(100, [0, 1])
> ```

#### getAmountOutPrice

---

###### 接口功能

> 查询卖出汇率

###### 请求参数

> | 参数      | 必选 | 类型         | 说明                               |
> | :-------- | :--- | :----------- | ---------------------------------- |
> | amount_in | ture | u128         | 输入量（swap操作左固定，查询右边） |
> | path      | true | Vec<AssetId> | 兑换路径                           |

###### 返回字段

> | 返回字段 | 字段类型 | 说明             |
> | :------- | :------- | :--------------- |
> | 数值     | Hex      | 计算得到的输出量 |

###### http请求示例

> ~~~
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>         "jsonrpc": "2.0",
>         "id": 1059,
>         "method": "swap_getAmountOutPrice",
>         "params": [100,[0, 1]]
>   }'
> ~~~

###### Response

> ```json
> {
>     "jsonrpc": "2.0",
>     "result": "0x63",
>     "id": 1059
> }
> ```

###### ws请求示例

> ```
> api.rpc.swap.getAmountOutPrice(100, [0, 1])
> ```

#### getTokenList

---

###### 接口功能

> 获取当前有流动性的所有Token相关信息

###### 请求参数

> 无

###### 返回字段

> | 返回字段  | 字段类型 | 说明      |
> | :-------- | :------- | :-------- |
> | assertId  | AssetId  | 资产id    |
> | chain     | string   | 链来源    |
> | decimals  | u8       | 精度      |
> | desc      | string   | 描述      |
> | token     | string   | token缩写 |
> | tokenName | string   | token名字 |

###### http请求示例

> ~~~
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>         "jsonrpc": "2.0",
>         "id": 1059,
>         "method": "swap_getTokenList"
>   }'
> ~~~

###### Response

> ```json
> {
>     "jsonrpc": "2.0",
>     "result": [
>         {
>             "assertId": 0,
>             "assertInfo": {
>                 "chain": "ChainX",
>                 "decimals": 8,
>                 "desc": "ChainX's crypto currency in Polkadot ecology",
>                 "token": "PCX",
>                 "tokenName": "Polkadot ChainX"
>             }
>         },
>         {
>             "assertId": 1,
>             "assertInfo": {
>                 "chain": "Bitcoin",
>                 "decimals": 8,
>                 "desc": "ChainX's Cross-chain Bitcoin",
>                 "token": "XBTC",
>                 "tokenName": "ChainX Bitcoin"
>             }
>         }
>     ],
>     "id": 1059
> }
> ```

###### ws请求示例

> ```
> api.rpc.swap.getTokenList()
> ```

#### getBalance

---

###### 接口功能

> 用户查询资产余额

###### 请求参数

> | 参数     | 必选 | 类型      | 说明   |
> | :------- | :--- | :-------- | ------ |
> | asset_id | ture | AssetId   | 资产id |
> | account  | true | AccountId | 账户   |

###### 返回字段

> | 返回字段 | 字段类型 | 说明     |
> | :------- | :------- | :------- |
> | 数值     | Hex      | 资产余额 |

###### http请求示例

> ~~~
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>      "jsonrpc": "2.0",
>      "id": 1059,
>      "method": "swap_getBalance",
>      "params": [0, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"]
>  }'
> ~~~

###### Response

> ```json
> {
>     "jsonrpc": "2.0",
>     "result": "0xde0b6b3a7640000",
>     "id": 1059
> }
> ```

###### ws请求示例

> ```
> api.rpc.swap.getBalance(100, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
> ```

## Extrinsics

#### createPair

###### 功能

> 创建资产对，必须创建后才能进行swap等交易

###### 参数

> | 参数    | 类型    | 类型         |
> | :------ | :------ | :----------- |
> | asset_0 | AssetId | 第一种资产id |
> | asset_1 | AssetId | 第二种资产id |

#### swapExactTokensForTokens

###### 功能

> swap操作,按照输入量进行swap交易

###### 参数

> | 参数           | 类型         | 类型                                 |
> | :------------- | :----------- | :----------------------------------- |
> | amount_in      | u128         | 输入的量                             |
> | amount_out_min | u128         | 执行交易时，最小输出量               |
> | path           | Vec<AssetId> | 兑换路径                             |
> | recipient      | AccountId    | 接收账户                             |
> | deadline       | BlockNumber  | 执行交易时，最大块高度，超过交易失败 |

#### swapTokensForExactTokens

###### 功能

> swap操作,按照输出量进行swap交易

###### 参数

> | 参数          | 类型         | 类型                                 |
> | :------------ | :----------- | :----------------------------------- |
> | amount_out    | u128         | 输出的量                             |
> | amount_in_max | u128         | 执行交易时，最大输入量               |
> | path          | Vec<AssetId> | 兑换路径                             |
> | recipient     | AccountId    | 接收账户                             |
> | deadline      | BlockNumber  | 执行交易时，最大块高度，超过交易失败 |

#### addLiquidity

###### 功能

> 增加流动性

###### 参数

> | 参数             | 类型        | 类型                                 |
> | :--------------- | :---------- | :----------------------------------- |
> | asset_0          | AssetId     | 第一种资产id                         |
> | asset_1          | AssetId     | 第二种资产id                         |
> | amount_0_desired | u128        | 第一种资产提供量                     |
> | amount_1_desired | u128        | 第二种资产提供量                     |
> | amount_0_min     | u128        | 第一种资产最小值                     |
> | amount_1_min     | u128        | 第二种资产最小值                     |
> | deadline         | BlockNumber | 执行交易时，最大块高度，超过交易失败 |

#### removeLiquidity

###### 功能

> 减少流动性

###### 参数

> | 参数               | 类型        | 说明             |
> | :----------------- | :---------- | :--------------- |
> | asset_0            | AssetId     | 第一种资产id     |
> | asset_1            | AssetId     | 第二种资产id     |
> | liquidity          | u128        | 流动性减少量     |
> | amount_asset_0_min | u128        | 第一种资产最小值 |
> | amount_asset_1_min | u128        | 第二种资产最小值 |
> | recipient          | AccountId   | 接收账户         |
> | deadline           | BlockNumber | 最大块高度       |

## Rpc Calls

~~~json
{
  "swap": {
    "getAmountInPrice": {
      "description": "Return amount in price by amount out",
      "params": [
        {
          "name": "amount_out",
          "type": "String"
        },
        {
          "name": "path",
          "type": "Vec<AssetId>"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "u128"
    },
    "getAmountOutPrice": {
      "description": "Return amount out price by amount in",
      "params": [
        {
          "name": "amount_in",
          "type": "String"
        },
        {
          "name": "path",
          "type": "Vec<AssetId>"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "u128"
    },
    "getTokenList":{
      "description": "Return all token list info",
      "params": [
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Vec<TokenInfo>"
    },
    "getBalance": {
      "description": "Return balance of (asset_id, who)",
      "params": [
        {
          "name": "asset_id",
          "type": "AssetId"
        },
        {
          "name": "account",
          "type": "AccountId"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "u128"
    }
  }
}
~~~

## Types

~~~json
{
  "AssetId": "u32",
  "TokenInfo": {
    "assertId": "AssetId",
    "assertInfo": "AssetInfo"
  },
  "AssetInfo": {
    "token": "String",
    "tokenName": "String",
    "chain": "Chain",
    "decimals": "Decimals",
    "desc": "String"
  },
  "Chain": {
    "_enum": [
      "ChainX",
      "Bitcoin",
      "Ethereum",
      "Polkadot"
    ]
  },
  "String": "Text",
  "Decimals": "u8",
  "AssetRestrictions": {"bits": "u32"},
  "AssetType": {
    "_enum": [
      "Usable",
      "Locked",
      "Reserved",
      "ReservedWithdrawal",
      "ReservedDexSpot"
    ]
  },
  "Desc": "Vec<u8>",
  "Token": "Vec<u8>",
  "BlockLength": "u32",
  "BlockWeights": {
    "baseBlock": "Weight",
    "maxBlock": "Weight",
    "perClass": "PerDispatchClass"
  },
  "PerDispatchClass": {
    "normal": "WeightPerClass",
    "operational": "WeightPerClass",
    "mandatory": "WeightPerClass"
  },
  "WeightPerClass": {
    "baseExtrinsic": "Weight",
    "maxExtrinsic": "Weight",
    "maxTotal": "Option<Weight>",
    "reserved": "Option<Weight>"
  },
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "RequestId": "u128",
  "BlockNumberFor": "BlockNumber",
  "Vault": {
    "id": "AccountId",
    "toBeIssuedTokens": "Balance",
    "issuedTokens": "Balance",
    "toBeRedeemedTokens": "Balance",
    "wallet": "Text",
    "bannedUntil": "BlockNumber",
    "status": "VaultStatus"
  },
  "VaultStatus": {
    "_enum": [
      "Active",
      "Liquidated",
      "CommittedTheft"
    ]
  },
  "TradingPrice": {
    "price": "u128",
    "decimal": "u8"
  },
  "AddrStr": "Text",
  "Network": {
    "_enum": [
      "Mainnet",
      "Testnet"
    ]
  },
  "AddressHash": "H160",
  "IssueRequest": {
    "vault": "AccountId",
    "openTime": "BlockNumber",
    "requester": "AccountId",
    "btcAddress": "BtcAddress",
    "completed": "bool",
    "cancelled": "bool",
    "btcAmount": "Balance",
    "griefingCollateral": "Balance"
  },
  "RedeemRequestStatus": {
    "_enum": [
      "Processing",
      "Cancled",
      "Completed"
    ]
  },
  "RedeemRequest": {
    "vault": "AccountId",
    "openTime": "BlockNumber",
    "requester": "AccountId",
    "btcAddress": "BtcAddress",
    "amount": "Balance",
    "redeemFee": "Balance",
    "status": "RedeemRequestStatus",
    "reimburse": "bool"
  },
  "chainbridge::ChainId": "u8",
  "ChainId": "u8",
  "ResourceId": "[u8; 32]",
  "DepositNonce": "u64",
  "ProposalVotes": {
    "votes_for": "Vec<AccountId>",
    "votes_against": "Vec<AccountId>",
    "status": "enum"
  },
  "Erc721Token": {
    "id": "TokenId",
    "metadata": "Vec<u8>"
  },
  "TokenId": "U256"
}
~~~

