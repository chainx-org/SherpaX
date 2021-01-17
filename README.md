How to use/test locally

### 1. Supported Polkadot dependencies

It should at least work until the following commits

-   rustc 1.49.0-nightly (beb5ae474 2020-10-04)
-   polkadot.js.org (apps v0.75.2-56)
-   Polkadot rococo-v1 @ `a3e1113655acfa42ae210310de90df086933ce26`
-   Cumulus rococo-v1 @ `7277f7aae5c0b45543a53d823d0c6c1da2a3ab18`
-   Substrate rococo-v1 (newer than 2.0.0) @ `a9bbc7bdbfd3fa66537e5feedf1562dcb2c132a5`

### 2. How to use

1. Spin up Polkadot validators (number of parachains + 1)
2. Spin up Collator(s)

Recommend checking out the [cumulus-workshop](https://substrate.dev/cumulus-workshop/#/3-parachains/1-launch) and following most of the steps described there, mainly 3.
Unfortunately, some commands there are outdated as the workshop has not been updated to the newest Rococo version, yet.
The following code is basically copied from there and updated to the new version to have a one-page-overview for all commands and steps.
Please check out the workshop for explanations.

### 3. Launch a local setup including a Relay Chain and a Parachain

#### Launch the Relay Chain

```bash
# Compile Polkadot with the real overseer feature
git clone https://github.com/paritytech/polkadot
git checkout rococo-v1
cargo build --release --features=real-overseer

# Generate a raw chain spec
./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode > rococo-custom-local.json
sed -i 's/"validation_upgrade_frequency": 600/"validation_upgrade_frequency": 10/g' rococo-custom-local.json
sed -i 's/"validation_upgrade_delay": 300/"validation_upgrade_delay": 5/g' rococo-custom-local.json
./target/release/polkadot build-spec --chain rococo-custom-local.json --disable-default-bootnode --raw > rococo-local-cfde-real-overseer.json

# Alice
./target/release/polkadot --chain rococo-local-cfde-real-overseer.json --alice --tmp

# Bob (In a separate terminal)
./target/release/polkadot --chain rococo-local-cfde-real-overseer.json --bob --tmp --port 30334
```

#### Launch the Parachain
`parachain-id` is u32 type. 
e.g `parachain-id=59`

```bash
# Compile
cargo build --release

# Export genesis state
# --parachain-id 59 as an example that can be chosen freely. Make sure to everywhere use the same parachain id
./target/release/sherpax export-genesis-state --parachain-id 59 > genesis-state

# Export genesis wasm
./target/release/sherpax export-genesis-wasm > genesis-wasm

# Collator1
./target/release/sherpax --collator --tmp --parachain-id 59 --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30335

# Collator2
./target/release/sherpax --collator --tmp --parachain-id 59 --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30336

# Parachain Full Node 1
./target/release/sherpax --tmp --parachain-id 59 --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30337
```

### 4. Register the parachain
![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)


### 5. Polkadot Apps Extrinsics Error

In case the apps complain about missing types when registering the parachain via a Polkadot validator, try to add the following:
```
{
  "Address": "AccountId",
  "LookupSource": "AccountId",
  "RefCount": "u32",
  "Keys": "(AccountId,AccountId,AccountId,AccountId,AccountId,AccountId)",
  "AccountInfo": "AccountInfoWithRefCount",
  "PairId": "u32",
  "Pair": {
    "token_0": "AssetId",
    "token_1": "AssetId",
    "account": "AccountId",
    "total_liquidity": "TokenBalance"
  },
  "PairInfo": {
    "token_0": "AssetId",
    "token_1": "AssetId",
    "account": "AccountId",
    "total_liquidity": "TokenBalance",
    "holding_liquidity": "TokenBalance"
  },
  "AssetId": {
    "_enum": {
      "NativeCurrency": null,
      "ParaCurrency": "u32"
    }
  },
  "Id": "u32",
  "CurrencyOf": "u128",
  "ExchangeId": "u32",
  "TokenBalance": "u64",
  "OriginKind": {
    "_enum": {
      "Native": null,
      "SovereignAccount": null,
      "Superuser": null
    }
  },
  "NetworkId": {
    "_enum": {
      "Any": null,
      "Named": "Vec<u8>",
      "Polkadot": null,
      "Kusama": null
    }
  },
  "MultiLocation": {
    "_enum": {
      "Null": null,
      "X1": "Junction",
      "X2": "(Junction, Junction)",
      "X3": "(Junction, Junction, Junction)",
      "X4": "(Junction, Junction, Junction, Junction)"
    }
  },
  "AccountId32Junction": {
    "network": "NetworkId",
    "id": "AccountId"
  },
  "AccountIndex64Junction": {
    "network": "NetworkId",
    "index": "Compact<u64>"
  },
  "AccountKey20Junction": {
    "network": "NetworkId",
    "index": "[u8; 20]"
  },
  "Junction": {
    "_enum": {
      "Parent": null,
      "Parachain": "Compact<u32>",
      "AccountId32": "AccountId32Junction",
      "AccountIndex64": "AccountIndex64Junction",
      "AccountKey20": "AccountKey20Junction",
      "PalletInstance": "u8",
      "GeneralIndex": "Compact<u128>",
      "GeneralKey": "Vec<u8>",
      "OnlyChild": null
    }
  },
  "VersionedMultiLocation": {
    "_enum": {
      "V0": "MultiLocation"
    }
  },
  "AssetInstance": {
    "_enum": {
      "Undefined": null,
      "Index8": "u8",
      "Index16": "Compact<u16>",
      "Index32": "Compact<u32>",
      "Index64": "Compact<u64>",
      "Index128": "Compact<u128>",
      "Array4": "[u8; 4]",
      "Array8": "[u8; 8]",
      "Array16": "[u8; 16]",
      "Array32": "[u8; 32]",
      "Blob": "Vec<u8>"
    }
  },
  "AbstractFungible": {
    "id": "Vec<u8>",
    "instance": "Compact<u128>"
  },
  "AbstractNonFungible": {
    "class": "Vec<u8>",
    "instance": "AssetInstance"
  },
  "ConcreteFungible": {
    "id": "MultiLocation",
    "amount": "Compact<u128>"
  },
  "ConcreteNonFungible": {
    "class": "MultiLocation",
    "instance": "AssetInstance"
  },
  "MultiAsset": {
    "_enum": {
      "None": null,
      "All": null,
      "AllFungible": null,
      "AllNonFungible": null,
      "AllAbstractFungible": "Vec<u8>",
      "AllAbstractNonFungible": "Vec<u8>",
      "AllConcreteFungible": "MultiLocation",
      "AllConcreteNonFungible": "MultiLocation",
      "AbstractFungible": "AbstractFungible",
      "AbstractNonFungible": "AbstractNonFungible",
      "ConcreteFungible": "ConcreteFungible",
      "ConcreteNonFungible": "ConcreteNonFungible"
    }
  },
  "VersionedMultiAsset": {
    "_enum": {
      "V0": "MultiAsset"
    }
  },
  "DepositAsset": {
    "assets": "Vec<MultiAsset>",
    "dest": "MultiLocation"
  },
  "DepositReserveAsset": {
    "assets": "Vec<MultiAsset>",
    "dest": "MultiLocation",
    "effects": "Vec<Order>"
  },
  "ExchangeAsset": {
    "give": "Vec<MultiAsset>",
    "receive": "Vec<MultiAsset>"
  },
  "InitiateReserveWithdraw": {
    "assets": "Vec<MultiAsset>",
    "reserve": "MultiLocation",
    "effects": "Vec<Order>"
  },
  "InitiateTeleport": {
    "assets": "Vec<MultiAsset>",
    "dest": "MultiLocation",
    "effects": "Vec<Order>"
  },
  "QueryHolding": {
    "query_id": "Compact<u64>",
    "dest": "MultiLocation",
    "assets": "Vec<MultiAsset>"
  },
  "Order": {
    "_enum": {
      "Null": null,
      "DepositAsset": "DepositAsset",
      "DepositReserveAsset": "DepositReserveAsset",
      "ExchangeAsset": "ExchangeAsset",
      "InitiateReserveWithdraw": "InitiateReserveWithdraw",
      "InitiateTeleport": "InitiateTeleport",
      "QueryHolding": "QueryHolding"
    }
  },
  "WithdrawAsset": {
    "assets": "Vec<MultiAsset>",
    "effects": "Vec<Order>"
  },
  "ReserveAssetDeposit": {
    "assets": "Vec<MultiAsset>",
    "effects": "Vec<Order>"
  },
  "TeleportAsset": {
    "assets": "Vec<MultiAsset>",
    "effects": "Vec<Order>"
  },
  "Balances": {
    "query_id": "Compact<u64>",
    "assets": "Vec<MultiAsset>"
  },
  "Transact": {
    "origin_type": "OriginKind",
    "call": "Vec<u8>"
  },
  "RelayTo": {
    "dest": "MultiLocation",
    "inner": "VersionedXcm"
  },
  "RelayedFrom": {
    "superorigin": "MultiLocation",
    "inner": "VersionedXcm"
  },
  "Xcm": {
    "_enum": {
      "WithdrawAsset": "WithdrawAsset",
      "ReserveAssetDeposit": "ReserveAssetDeposit",
      "TeleportAsset": "TeleportAsset",
      "Balances": "Balances",
      "Transact": "Transact",
      "RelayTo": "RelayTo",
      "RelayedFrom": "RelayedFrom"
    }
  },
  "VersionedXcm": {
    "_enum": {
      "V0": "Xcm"
    }
  },
  "XcmError": {
    "_enum": [
      "Undefined",
      "Unimplemented",
      "UnhandledXcmVersion",
      "UnhandledXcmMessage",
      "UnhandledEffect",
      "EscalationOfPrivilege",
      "UntrustedReserveLocation",
      "UntrustedTeleportLocation",
      "DestinationBufferOverflow",
      "CannotReachDestination",
      "MultiLocationFull",
      "FailedToDecode",
      "BadOrigin"
    ]
  },
  "XcmResult": {
    "_enum": {
      "Ok": "()",
      "Err": "XcmError"
    }
  },
  "HrmpChannelId": {
    "sender": "u32",
    "receiver": "u32"
  },
  "AvailabilityBitfield": "BitVec",
  "SignedAvailabilityBitfield": {
    "payload": "BitVec",
    "validator_index": "u32",
    "signature": "Signature"
  },
  "SignedAvailabilityBitfields": "Vec<SignedAvailabilityBitfield>",
  "ValidatorSignature": "Signature",
  "HeadData": "Vec<u8>",
  "CandidateDescriptor": {
    "para_id": "u32",
    "relay_parent": "Hash",
    "collator_id": "Hash",
    "persisted_validation_data_hash": "Hash",
    "pov_hash": "Hash",
    "erasure_root": "Hash",
    "signature": "Signature"
  },
  "CandidateReceipt": {
    "descriptor": "CandidateDescriptor",
    "commitments_hash": "Hash"
  },
  "UpwardMessage": "Vec<u8>",
  "OutboundHrmpMessage": {
    "recipient": "u32",
    "data": "Vec<u8>"
  },
  "ValidationCode": "Vec<u8>",
  "CandidateCommitments": {
    "upward_messages": "Vec<UpwardMessage>",
    "horizontal_messages": "Vec<OutboundHrmpMessage>",
    "new_validation_code": "Option<ValidationCode>",
    "head_data": "HeadData",
    "processed_downward_messages": "u32",
    "hrmp_watermark": "BlockNumber"
  },
  "CommittedCandidateReceipt": {
    "descriptor": "CandidateDescriptor",
    "commitments": "CandidateCommitments"
  },
  "ValidityAttestation": {
    "_enum": {
      "DummyOffsetBy1": "Raw",
      "Implicit": "ValidatorSignature",
      "Explicit": "ValidatorSignature"
    }
  },
  "BackedCandidate": {
    "candidate": "CommittedCandidateReceipt",
    "validity_votes": "Vec<ValidityAttestation>",
    "validator_indices": "BitVec"
  },
  "CandidatePendingAvailablility": {
    "core": "u32",
    "descriptor": "CandidateDescriptor",
    "availability_votes": "BitVec",
    "relay_parent_number": "BlockNumber",
    "backed_in_number": "BlockNumber"
  },
  "BufferedSessionChange": {
    "apply_at": "BlockNumber",
    "validators": "Vec<ValidatorId>",
    "queued": "Vec<ValidatorId>",
    "session_index": "SessionIndex"
  },
  "HostConfiguration": {
    "max_code_size": "u32",
    "max_head_data_size": "u32",
    "max_upward_queue_count": "u32",
    "max_upward_queue_size": "u32",
    "max_upward_message_size": "u32",
    "max_upward_message_num_per_candidate": "u32",
    "hrmp_max_message_num_per_candidate": "u32",
    "validation_upgrade_frequency": "u32",
    "validation_upgrade_delay": "u32",
    "max_pov_size": "u32",
    "max_downward_message_size": "u32",
    "preferred_dispatchable_upward_messages_step_weight": "Weight",
    "hrmp_max_parachain_outbound_channels": "u32",
    "hrmp_max_parathread_outbound_channels": "u32",
    "hrmp_open_request_ttl": "u32",
    "hrmp_sender_deposit": "Balance",
    "hrmp_recipient_deposit": "Balance",
    "hrmp_channel_max_capacity": "u32",
    "hrmp_channel_max_total_size": "u32",
    "hrmp_max_parachain_inbound_channels": "u32",
    "hrmp_max_parathread_inbound_channels": "u32",
    "hrmp_channel_max_message_size": "u32",
    "acceptance_period": "u32",
    "parathread_cores": "u32",
    "parathread_retries": "u32",
    "group_rotation_frequency": "u32",
    "chain_availability_period": "u32",
    "thread_availability_period": "u32",
    "scheduling_lookahead": "u32",
    "max_validators_per_core": "Option<u32>",
    "dispute_period": "u32",
    "no_show_slots": "u32",
    "n_delay_tranches": "u32",
    "zeroth_delay_tranche_width": "u32",
    "needed_approvals": "u32",
    "relay_vrf_modulo_samples": "u32"
  },
  "InboundDownwardMessage": {
    "sent_at": "u32",
    "msg": "Vec<u8>"
  },
  "InboundHrmpMessage": {
    "sent_at": "u32",
    "data": "Vec<u8>"
  },
  "MessageIngestionType": {
    "dmp": "Vec<InboundDownwardMessage>",
    "hrmp": "BTreeMap<u32, Vec<InboundHrmpMessage>>"
  },
  "HrmpChannel": {
    "max_capacity": "u32",
    "max_total_size": "u32",
    "max_message_size": "u32",
    "msg_count": "u32",
    "total_size": "u32",
    "mqc_head": "Option<Hash>",
    "sender_deposit": "Balance",
    "recipient_deposit": "Balance"
  },
  "PersistedValidationData": {
    "parent_head": "HeadData",
    "block_number": "BlockNumber",
    "relay_storage_root": "Hash",
    "hrmp_mqc_heads": "Vec<(Id, Hash)>",
    "dmq_mqc_head": "Hash",
    "max_pov_size": "u32"
  },
  "TransientValidationData": {
    "max_code_size": "u32",
    "max_head_data_size": "u32",
    "balance": "Balance",
    "code_upgrade_allowed": "Option<BlockNumber>",
    "dmq_length": "u32"
  },
  "ValidationData": {
    "persisted": "PersistedValidationData",
    "transient": "TransientValidationData"
  },
  "StorageProof": {
    "trie_nodes": "Vec<Vec<u8>>"
  },
  "ValidationDataType": {
    "validation_data": "ValidationData",
    "relay_chain_state": "StorageProof"
  }
}
```
