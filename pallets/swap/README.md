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
> | assetId  | AssetId  | 资产id    |
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
>             "assetId": 0,
>             "assetInfo": {
>                 "chain": "ChainX",
>                 "decimals": 8,
>                 "desc": "ChainX's crypto currency in Polkadot ecology",
>                 "token": "PCX",
>                 "tokenName": "Polkadot ChainX"
>             }
>         },
>         {
>             "assetId": 1,
>             "assetInfo": {
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

#### getAllPairs

---

###### 接口功能

> 获取当前有流动性的所有pair对

###### 请求参数

> 无

###### 返回字段

> | 返回字段 | 字段类型                | 说明       |
> | :------- | :---------------------- | :--------- |
> |          | Vec[(AssetId, AssetId)] | pair对列表 |

###### http请求示例

> ~~~
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>      "jsonrpc": "2.0",
>      "id": 1059,
>      "method": "swap_getAllPairs"
>  }'
> ~~~

###### Response

> ```json
> {
>     "jsonrpc": "2.0",
>     "result": [
>         [
>             0,
>             3221225473
>         ],
>         [
>             0,
>             60
>         ]
>     ],
>     "id": 1059
> }
> ```

###### ws请求示例

> ```
> api.rpc.swap.getAllPairs()
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
          "type": "u128"
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
          "type": "u128"
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
    },
    "getAllPairs":{
      "description": "Return all pairs",
      "params": [
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Vec<(AssetId, AssetId)>"
    }
  }
}
~~~

## Types

~~~json
{
  "AssetId": "u32",
  "TokenInfo": {
    "assetId": "AssetId",
    "assetInfo": "AssetInfo"
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
  "AssetRestrictions": {
    "bits": "u32"
  },
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
  "Amount": "i128",
  "AmountOf": "Amount",
  "CurrencyIdOf": "AssetId",
  "CurrencyId": "AssetId",
  "AssetRestriction": {
    "_enum": [
      "Move",
      "Transfer",
      "Deposit",
      "Withdraw",
      "DestroyWithdrawal",
      "DestroyFree"
    ]
  },
  "Handicap": {
    "highest_bid": "Price",
    "lowest_ask": "Price"
  },
  "NetworkType": {
    "_enum": [
      "Mainnet",
      "Testnet"
    ]
  },
  "Order": {
    "props": "OrderProperty",
    "status": "OrderStatus",
    "remaining": "Balance",
    "executed_indices": "Vec<TradingHistoryIndex>",
    "already_filled": "Balance",
    "last_update_at": "BlockNumber"
  },
  "OrderProperty": {
    "id": "OrderId",
    "side": "Side",
    "price": "Price",
    "amount": "Amount",
    "pair_id": "TradingPairId",
    "submitter": "AccountId",
    "order_type": "OrderType",
    "created_at": "BlockNumber"
  },
  "TotalAssetInfo": {
    "info": "AssetInfo",
    "balance": "BTreeMap<AssetType, Balance>",
    "is_online": "bool",
    "restrictions": "AssetRestrictions"
  },
  "NominatorLedger": {
    "nomination": "Balance",
    "last_vote_weight": "VoteWeight",
    "last_vote_weight_update": "BlockNumber",
    "unbonded_chunks": "Vec<Unbonded>"
  },
  "Unbonded": {
    "value": "Balance",
    "locked_until": "BlockNumber"
  },
  "WithdrawalRecordId": "u32",
  "WithdrawalState": {
    "_enum": [
      "Applying",
      "Processing",
      "NormalFinish",
      "RootFinish",
      "NormalCancel",
      "RootCancel"
    ]
  },
  "WithdrawalRecord": {
    "asset_id": "AssetId",
    "applicant": "AccountId",
    "balance": "Balance",
    "addr": "AddrStr",
    "ext": "Memo",
    "height": "BlockNumber"
  },
  "WithdrawalLimit": {
    "minimal_withdrawal": "Balance",
    "fee": "Balance"
  },
  "TrusteeInfoConfig": {
    "min_trustee_count": "u32",
    "max_trustee_count": "u32"
  },
  "GenericTrusteeIntentionProps": {
    "about": "Text",
    "hot_entity": "Vec<u8>",
    "cold_entity": "Vec<u8>"
  },
  "GenericTrusteeSessionInfo": {
    "trustee_list": "Vec<AccountId>",
    "threshold": "u16",
    "hot_address": "Vec<u8>",
    "cold_address": "Vec<u8>"
  },
  "ChainAddress": "Vec<u8>",
  "BtcTrusteeType": "Vec<u8>",
  "BtcTrusteeAddrInfo": {
    "addr": "BtcAddress",
    "redeem_script": "Vec<u8>"
  },
  "BtcTrusteeIntentionProps": {
    "about": "Text",
    "hot_entity": "BtcTrusteeType",
    "cold_entity": "BtcTrusteeType"
  },
  "BtcTrusteeSessionInfo": {
    "trustee_list": "Vec<AccountId>",
    "threshold": "u16",
    "hot_address": "BtcTrusteeAddrInfo",
    "cold_address": "BtcTrusteeAddrInfo"
  },
  "BtcNetwork": {
    "_enum": [
      "Mainnet",
      "Testnet"
    ]
  },
  "BtcAddress": "Text",
  "BtcHeader": "Vec<u8>",
  "BtcTransaction": "Vec<u8>",
  "BtcPartialMerkleTree": "Vec<u8>",
  "BtcRelayedTxInfo": {
    "block_hash": "H256",
    "merkle_proof": "BtcPartialMerkleTree"
  },
  "BtcHeaderIndex": {
    "hash": "H256",
    "height": "u32"
  },
  "BtcTxResult": {
    "_enum": [
      "Success",
      "Failure"
    ]
  },
  "BtcTxState": {
    "tx_type": "BtcTxType",
    "result": "BtcTxResult"
  },
  "BtcTxType": {
    "_enum": [
      "Withdrawal",
      "Deposit",
      "HotAndCold",
      "TrusteeTransition",
      "Irrelevance"
    ]
  },
  "BtcDepositCache": {
    "txid": "H256",
    "balance": "u64"
  },
  "BtcVoteResult": {
    "_enum": [
      "Unfinish",
      "Finish"
    ]
  },
  "BtcWithdrawalProposal": {
    "sig_state": "BtcVoteResult",
    "withdrawal_id_list": "Vec<u32>",
    "tx": "BtcTransaction",
    "trustee_list": "Vec<(AccountId, bool)>"
  },
  "BtcTxVerifier": {
    "_enum": [
      "Recover",
      "RuntimeInterface"
    ]
  },
  "RpcTotalAssetInfo": {
    "info": "AssetInfo",
    "balance": "BTreeMap<AssetType, RpcBalance>",
    "is_online": "bool",
    "restrictions": "AssetRestrictions"
  },
  "RpcOrder": {
    "id": "OrderId",
    "side": "Side",
    "price": "RpcPrice",
    "amount": "RpcBalance",
    "pair_id": "TradingPairId",
    "submitter": "AccountId",
    "order_type": "OrderType",
    "created_at": "BlockNumber",
    "status": "OrderStatus",
    "remaining": "RpcBalance",
    "executed_indices": "Vec<TradingHistoryIndex>",
    "already_filled": "RpcBalance",
    "reserved_balance": "RpcBalance",
    "last_update_at": "BlockNumber"
  },
  "RpcWithdrawalRecord": {
    "asset_id": "AssetId",
    "applicant": "AccountId",
    "balance": "RpcBalance",
    "addr": "String",
    "ext": "String",
    "height": "BlockNumber",
    "state": "WithdrawalState"
  },
  "RpcMiningDividendInfo": {
    "own": "RpcBalance",
    "other": "RpcBalance",
    "insufficient_stake": "RpcBalance"
  },
  "RpcInclusionFee": {
    "base_fee": "RpcBalance",
    "len_fee": "RpcBalance",
    "adjusted_weight_fee": "RpcBalance"
  },
  "RpcFeeDetails": {
    "inclusion_fee": "Option<RpcInclusionFee>",
    "tip": "RpcBalance",
    "extra_fee": "RpcBalance",
    "final_fee": "RpcBalance"
  },
  "ValidatorInfo": {
    "account": "AccountId",
    "registered_at": "BlockNumber",
    "is_chilled": "bool",
    "last_chilled": "Option<BlockNumber>",
    "total_nomination": "RpcBalance",
    "last_total_vote_weight": "RpcVoteWeight",
    "last_total_vote_weight_update": "BlockNumber",
    "is_validating": "bool",
    "self_bonded": "RpcBalance",
    "referral_id": "String",
    "reward_pot_account": "AccountId",
    "reward_pot_balance": "RpcBalance"
  },
  "FullPairInfo": {
    "base_currency": "AssetId",
    "highest_bid": "RpcPrice",
    "id": "TradingPairId",
    "latest_price": "RpcPrice",
    "latest_price_updated_at": "BlockNumber",
    "lowest_ask": "RpcPrice",
    "max_valid_bid": "RpcPrice",
    "min_valid_ask": "RpcPrice",
    "pip_decimals": "u32",
    "quote_currency": "AssetId",
    "tick_decimals": "u32",
    "tradable": "bool"
  },
  "MiningAssetInfo": {
    "asset_id": "AssetId",
    "mining_power": "FixedAssetPower",
    "reward_pot": "AccountId",
    "reward_pot_balance": "RpcBalance",
    "last_total_mining_weight": "RpcMiningWeight",
    "last_total_mining_weight_update": "BlockNumber"
  },
  "Depth": {
    "asks": "Vec<(RpcPrice, RpcBalance)>",
    "bids": "Vec<(RpcPrice, RpcBalance)>"
  },
  "Page": {
    "page_index": "u32",
    "page_size": "u32",
    "data": "Vec<RpcOrder>"
  },
  "Price": "u128",
  "Balance": "u128",
  "MiningWeight": "u128",
  "VoteWeight": "u128",
  "RpcPrice": "String",
  "RpcBalance": "String",
  "RpcMiningWeight": "String",
  "RpcVoteWeight": "String",
  "OrderInfo": "Order",
  "HandicapInfo": "Handicap",
  "FullIdentification": "ValidatorId",
  "WithdrawalRecordOf": "WithdrawalRecord",
  "ChainId": "u8",
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
  "TokenId": "U256",
  "BtcHeaderInfo": {
    "header": "BtcHeader",
    "height": "u32"
  },
      "BtcParams": {
        "maxBits": "u32",
        "blockMaxFuture": "u32",
        "targetTimespanSeconds": "u32",
        "targetSpacingSeconds": "u32",
        "retargetingFactor": "u32",
        "retargetingInterval": "u32",
        "minTimespan": "u32",
        "maxTimespan": "u32"
    },
  "Memo": "Text"
}

~~~

