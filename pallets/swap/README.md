# Uniswap接口文档

## 类型映射

> | Balance     | u128 |
> | :---------- | :--- |
> | AssetId     | u32  |
> | BlockNumber | u32  |

## RPC

#### getAmountInPrice

---

###### 接口功能
> 查询买入汇率

###### URL
> http://localhost:6666

###### 支持格式
> JSON

###### HTTP请求方式
> POST　

###### 请求参数
> | 参数       | 必选 | 类型         | 说明                                 |
> | :--------- | :--- | :----------- | ------------------------------------ |
> | amount_out | ture | Balance      | 输出量（swap操作右边固定，查询左边） |
> | path       | true | Vec<AssetId> | 兑换路径                             |

###### 返回字段
> | 返回字段   | 字段类型 | 说明             |
> | :--------- | :------- | :--------------- |
> | 16进制数值 | Hex      | 计算得到的输入量 |

###### 接口示例
> ~~~apl
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>      "jsonrpc": "2.0",
>      "id": 1059,
>      "method": "getAmountInPrice",
>      "params": [100,[0, 1]]
>  }'
> ~~~
###### Response

> ```json
> {
>     "jsonrpc": "2.0",
>     "result": "0x64",
>     "id": 1059
> }
> ```

#### getAmountOutPrice

---

###### 接口功能

> 查询卖出汇率

###### URL

> http://localhost:6666

###### 支持格式

> JSON

###### HTTP请求方式

> POST　

###### 请求参数

> | 参数      | 必选 | 类型         | 说明                               |
> | :-------- | :--- | :----------- | ---------------------------------- |
> | amount_in | ture | Balance      | 输入量（swap操作左固定，查询右边） |
> | path      | true | Vec<AssetId> | 兑换路径                           |

###### 返回字段

> | 返回字段   | 字段类型 | 说明             |
> | :--------- | :------- | :--------------- |
> | 16进制数值 | Hex      | 计算得到的输出量 |

###### 接口示例

> ~~~apl
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>      "jsonrpc": "2.0",
>      "id": 1059,
>      "method": "getAmountOutPrice",
>      "params": [100,[0, 1]]
>  }'
> ~~~

###### Response

> ```json
> {
>     "jsonrpc": "2.0",
>     "result": "0x63",
>     "id": 1059
> }
> ```

#### getTokenList

---

###### 接口功能

> 获取当前有流动性的所有Token相关信息

###### URL

> http://localhost:6666

###### 支持格式

> JSON

###### HTTP请求方式

> POST　

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

###### 接口示例

> ~~~apl
> curl --location --request POST '127.0.0.1:6666' \
> --header 'Content-Type: application/json' \
> --data-raw '{
>      "jsonrpc": "2.0",
>      "id": 1059,
>      "method": "getTokenList"
>  }'
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

## Extrinsics

#### createPair

###### 功能

> 创建资产对，必须创建才能进行swap等交易

###### 参数

> | 参数    | 类型    | 类型         |
> | :------ | :------ | :----------- |
> | asset_0 | AssetId | 第一种资产id |
> | asset_0 | AssetId | 第二种资产id |

#### swapExactTokensForTokens

###### 功能

> swap正向操作,按照输入量进行swap交易

###### 参数

> | 参数           | 类型         | 类型                                   |
> | :------------- | :----------- | :------------------------------------- |
> | amount_in      | Balance      | 输入的量                               |
> | amount_out_min | Balance      | 执行交易时，输出量最小值，小于交易失败 |
> | path           | Vec<AssetId> | 兑换路径                               |
> | recipient      | Account      | 接收账户                               |
> | deadline       | BlockNumber  | 执行交易时，最大块高度，超过交易失败   |

#### swapTokensForExactTokens

###### 功能

> swap逆向操作,按照输出量进行swap交易

###### 参数

> | 参数          | 类型         | 类型                                   |
> | :------------ | :----------- | :------------------------------------- |
> | amount_out    | Balance      | 输出的量                               |
> | amount_in_max | Balance      | 执行交易时，输入量最大值，超过交易失败 |
> | path          | Vec<AssetId> | 兑换路径                               |
> | recipient     | Account      | 接收账户                               |
> | deadline      | BlockNumber  | 执行交易时，最大块高度，超过交易失败   |

#### addLiquidity

###### 功能

> 增加流动性

###### 参数

> | 参数             | 类型        | 类型                                 |
> | :--------------- | :---------- | :----------------------------------- |
> | asset_0          | AssetId     | 第一种资产id                         |
> | asset_1          | AssetId     | 第二种资产id                         |
> | amount_0_desired | Balance     | 第一种资产提供量                     |
> | amount_1_desired | Balance     | 第二种资产提供量                     |
> | amount_0_min     | Balance     | 第一种资产最小值                     |
> | amount_1_min     | Balance     | 第二种资产最小值                     |
> | deadline         | BlockNumber | 执行交易时，最大块高度，超过交易失败 |

#### removeLiquidity

###### 功能

> 减少流动性

###### 参数

> | 参数               | 类型        | 类型             |
> | :----------------- | :---------- | :--------------- |
> | asset_0            | AssetId     | 第一种资产id     |
> | asset_1            | AssetId     | 第二种资产id     |
> | liquidity          | Balance     | 流动性减少量     |
> | amount_asset_0_min | Balance     | 第一种资产最小值 |
> | amount_asset_1_min | Balance     | 第二种资产最小值 |
> | recipient          | Account     | 接收账户         |
> | deadline           | BlockNumber | 最大块高度       |

