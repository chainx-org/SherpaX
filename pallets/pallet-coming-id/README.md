# pallet-coming-id

## Overview
`pallet-coming-id` 是 `MinixChain` 上用来标识`Coming App`的用户身份的链上代理系统. 

每个`coming-id`可绑定指定类型的数据(如btc地址,eth地址,合约地址,传统互联网帐号等).

每个`coming-id`同一时间有且只有一个属主(`Substrate公私钥体系`).

## Intro
- `pallet-coming-id`(简称`cid`)由1-12位数字组成

   [1,100000)为`Coming`内部预留, 
   
   [100000,1000000) 为`Coming`社区预留, 
   
   [1000000,100000000000)所有用户均可申领.

- `cid`的分配权和转移权:  
  - 分配权: 
  
    `Coming`有所有cid的分配权;
    
  - 转移权: 
  
    `Coming`只拥有[1,100000)的转移权;
    
    其余cid的转移权归其属主拥有.
    
- 关键函数

  - register(cid, recipient): 

    highAdmin 权限, 分配 1-12位 cid.

    mediaAdmin 权限, 分配 6-12位 cid.

    lowAdmin 权限, 分配 7-12位 cid.
  
  - bond(cid, bond_data)
  
      user权限(owner), 对指定cid, bond数据(类型字段和数据字段):
  
      ```rust
      pub struct BondData {
         pub bond_type: BondType,
         pub data: Vec<u8>
      }
      ```
  
  - unbond(cid, bond_type)
   
      user权限(owner), unbond 指定cid, bond类型字段

## rpc
- get_account_id:
 获取指定cid的account id

```
#[rpc(name = "get_account_id")]
fn get_account_id(
   &self,
   cid: Cid,
   at: Option<BlockHash>
) -> Result<Option<AccountId>>;
```
输入：
```json
{
  "jsonrpc":"2.0",
  "id":1,
  "method":"get_account_id",
  "params": [1000000]
}
```
输出1：
```json
{
  "jsonrpc": "2.0",
  "result": null,
  "id": 1
}
```
输出2：
```json
{
  "jsonrpc": "2.0",
  "result": "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL",
  "id": 1
}
```
- get_cids:
 获取指定account id的cids
```
#[rpc(name = "get_cids")]
fn get_cids(
   &self,
   account: AccountId,
   at: Option<BlockHash>
) -> Result<Vec<Cid>>;
```
输入：
```json
{
  "jsonrpc":"2.0",
  "id":1,
  "method":"get_cids",
  "params": ["5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL"]
}
```
输出：
```json
{
  "jsonrpc": "2.0",
  "result": [
    99,
    999
  ],
  "id": 1
}
```
- get_bond_data:
 获取指定cid的bond data

```
#[rpc(name = "get_bond_data")]
fn get_bond_data(
    &self,
    cid: Cid,
    at: Option<BlockHash>
) -> Result<Option<CidDetails<AccountId>>>;
```
输入：
```json
{
  "jsonrpc":"2.0",
  "id":1,
  "method":"get_bond_data",
  "params": [99]
}
```
输出：
```json
{
  "jsonrpc": "2.0",
  "result": {
    "bonds": [
      {
        "bondType": 1,
        "data": "0x7b226e616d65223a227465737432227d"
      }
    ],
    "card": "0x7b226e616d65223a202274657374227d",
    "owner": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
  },
  "id": 1
}
```

- get_card:
 获取指定cid的c-card

```
#[rpc(name = "get_card")]
fn get_card(
    &self,
    cid: Cid,
    at: Option<BlockHash>
) -> Result<Option<CidDetails<AccountId>>>;
```
输入：
```json
{
  "jsonrpc":"2.0",
  "id":1,
  "method":"get_card",
  "params": [99]
}
```
输出：
```json
{
  "jsonrpc": "2.0",
  "result": "0x7b226e616d65223a202274657374227d",
  "id": 1
}
```

## custom types
```json
{
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "Cid": "u64",
  "BondType": "u16",
  "BondData": {
    "bond_type": "BondType",
    "data": "Bytes"
  },
  "CidDetails": {
    "owner": "AccountId",
    "bonds": "Vec<BondData>",
    "card":  "Bytes"
  }
}
```
## rpc custom 
```json
    
{
  "comingId": {
    "getAccountId": {
      "description": "comingId getAccountId",
      "params": [
        {
          "name": "cid",
          "type": "Cid"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Option<AccountId>"
    },
    "getCids": {
      "description": "comingId getCids",
      "params": [
        {
          "name": "account",
          "type": "AccountID"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Vec<Cid>"
    },
    "getBondData": {
      "description": "comingId getBondData",
      "params": [
        {
          "name": "cid",
          "type": "Cid"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Option<CidDetails>"
    },
    "getCard": {
      "description": "comingId getCard",
      "params": [
        {
          "name": "cid",
          "type": "Cid"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Option<Bytes>"
    }
  }
}
```
