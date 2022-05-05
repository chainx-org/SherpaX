// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_runtime::{traits::StaticLookup, AccountId32};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use xp_gateway_dogecoin::DogeTxType;
use xpallet_gateway_records::{Pallet as XGatewayRecords, WithdrawalState};

use light_bitcoin::{
    chain::{BlockHeader, Transaction},
    merkle::PartialMerkleTree,
    primitives::H256,
    serialization::{self, Reader, SERIALIZE_TRANSACTION_WITNESS},
};

use crate::{types::*, Call, Config, Pallet, PendingDeposits, TxState, WithdrawalProposal};

fn create_default_asset<T: Config>(who: T::AccountId) {
    let miner = T::Lookup::unlookup(who);
    let _ = pallet_assets::Pallet::<T>::force_create(
        RawOrigin::Root.into(),
        T::DogeAssetId::get(),
        miner,
        true,
        1u32.into(),
    );
}

fn generate_blocks_3782200_3782230() -> BTreeMap<u32, BlockHeader> {
    let bytes = include_bytes!("./res/headers-3782200-3782230.raw");
    Decode::decode(&mut &bytes[..]).unwrap()
}

fn account<T: Config>(pubkey: &str) -> T::AccountId {
    let pubkey = hex::decode(pubkey).unwrap();
    let mut public = [0u8; 32];
    public.copy_from_slice(pubkey.as_slice());
    let account = AccountId32::from(public).encode();
    Decode::decode(&mut account.as_slice()).unwrap()
}

fn alice<T: Config>() -> T::AccountId {
    // sr25519 Alice
    account::<T>("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
}

// fn bob<T: Config>() -> T::AccountId {
//     // sr25519 Bob
//     account::<T>("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")
// }

fn withdraw_tx() -> (Transaction, Vec<u8>, Transaction) {
    // block height: 3782224
    // https://signet.bitcoinexplorer.org/tx/0f592933b493bedab209851cb2cf07871558ff57d86d645877b16651479b51a2
    const RAW_TX: &str = "01000000019926808d419bbe850e9b6347146a3b6107381f6d075297acb969838d325673e800000000fd5c010047304402205a546a45118fdfeff5abb6470cfcda5ce8927227ff5aabacae2a3dda9a46ff900220574ca3ef630b6eb15e0e4744c0f4804452406a22e61639f5c320de78ba400b600147304402206a575b863f66dc69bc2104f117c354b18cdd8f612edb582a506a5b141fa2a74e02204318df9774970f0cbef1224821fa7dfecbc62949a4db48c3d8f0ee9eeafef448014cc95241042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3410451e0dc3d9709d860c49785fc84b62909d991cffd81592f6994c452438f91b6a2e586541c4b3bc1ebeb5fb9fad2ed2e696b2175c54458ab6f103717cbeeb4e52c4104a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b853ae000000000240420f00000000001976a9144da9bb5dea4c42219a2a120523d1a0ce6c268f3788ac00127a000000000017a9142995ac346d93b015e2941715d432af5ac4e1010c8700000000";
    let tx = RAW_TX.parse::<Transaction>().unwrap();

    // https://signet.bitcoinexplorer.org/tx/8e5d37c768acc4f3e794a10ad27bf0256237c80c22fa67117e3e3e1aec22ea5f
    const RAW_TX_PREV: &str = "010000000143fb4694093a57cd727791deac22563e1f6595b8f5dc519be4e8701b8afecec4000000008a47304402205ef330d36268379c78e32cfc3b04b3bfc8d595c9c161b65a9e81f866331dbdee02206c0e960eeeb74ea02deac4328251f5a62b39b185aa5a451134b77e873619f123014104a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b80000000002809698000000000017a9142995ac346d93b015e2941715d432af5ac4e1010c870000000000000000326a3035516a706f3772516e7751657479736167477a6334526a376f737758534c6d4d7141754332416255364c464646476a3800000000";
    let prev_tx = RAW_TX_PREV.parse::<Transaction>().unwrap();

    const RAW_PROOF: &str = "0200000002e808e8e91a23fb32ecbdd829105f789a030de599cb3c185775f4080101ef661a4a892af99a2b4b1a2a4bf157832c3870b26576e844ae1be2dfa962c02d8d72550105";
    let proof = hex::decode(RAW_PROOF).unwrap();
    let merkle_proof: PartialMerkleTree = serialization::deserialize(Reader::new(&proof)).unwrap();

    let header = generate_blocks_3782200_3782230()[&3782224];
    let info = DogeRelayedTxInfo {
        block_hash: header.hash(),
        merkle_proof,
    };
    (tx, info.encode(), prev_tx)
}

// push header 3782200 - 63310
fn prepare_headers<T: Config>(caller: &T::AccountId) {
    for (height, header) in generate_blocks_3782200_3782230() {
        if height == 3782200 {
            continue;
        }
        let header = serialization::serialize(&header).into();
        Pallet::<T>::push_header(RawOrigin::Signed(caller.clone()).into(), header).unwrap();
    }
}

benchmarks! {
    push_header {
        let receiver: T::AccountId = whitelisted_caller();
        let insert_height = 3782200 + 1;
        let header = generate_blocks_3782200_3782230()[&insert_height];
        let hash = header.hash();
        let header_raw = serialization::serialize(&header).into();
    }: _(RawOrigin::Signed(receiver), header_raw)
    verify {
        assert!(Pallet::<T>::headers(&hash).is_some());
    }

    push_transaction {
        let n = 1024 * 1024 * 500; // 500KB length
        let l = 1024 * 1024 * 500; // 500KB length

        let caller: T::AccountId = alice::<T>();
        create_default_asset::<T>(caller.clone());
        prepare_headers::<T>(&caller);
        let (tx, info, prev_tx) = withdraw_tx();
        let tx_hash = tx.hash();
        let tx_raw = serialization::serialize_with_flags(&tx, SERIALIZE_TRANSACTION_WITNESS).into();
        let prev_tx_raw = serialization::serialize_with_flags(&prev_tx, SERIALIZE_TRANSACTION_WITNESS).into();

        let amount: T::Balance = 1_000_000_000u32.into();
        let withdrawal = 1_000_000u32.into();

        XGatewayRecords::<T>::deposit(&caller, T::DogeAssetId::get(), amount).unwrap();
        XGatewayRecords::<T>::withdraw(&caller, T::DogeAssetId::get(), withdrawal, b"nbGodDo7pezD2LcKN8AFMc9nMPvT1YhXcc".to_vec(), b"".to_vec().into()).unwrap();

        XGatewayRecords::<T>::withdrawal_state_insert(0, WithdrawalState::Processing);

        let proposal = DogeWithdrawalProposal::<T::AccountId> {
            sig_state: VoteResult::Finish,
            withdrawal_id_list: vec![0],
            tx,
            trustee_list: vec![],
        };
        WithdrawalProposal::<T>::put(proposal);

    }: _(RawOrigin::Signed(caller), tx_raw, info, Some(prev_tx_raw))
    verify {
        assert!(WithdrawalProposal::<T>::get().is_none());
        assert_eq!(
            TxState::<T>::get(tx_hash),
            Some(DogeTxState {
                tx_type: DogeTxType::Withdrawal,
                result: DogeTxResult::Success,
            })
        );
    }

    create_dogecoin_withdraw_tx {
        let n = 100;                // 100 withdrawal count
        let l = 1024 * 1024 * 500;  // 500KB length

        let caller = alice::<T>();
        create_default_asset::<T>(caller.clone());

        let (tx, info, prev_tx) = withdraw_tx();
        let tx_hash = tx.hash();
        let tx_raw: Vec<u8> = serialization::serialize_with_flags(&tx, SERIALIZE_TRANSACTION_WITNESS).into();

        let amount: T::Balance = 1_000_000_000u32.into();

        let withdrawal: T::Balance = 1_000_000u32.into();

        #[cfg(feature = "runtime-benchmarks")]
        let withdrawal: T::Balance = 1_000_000u32.into();

        XGatewayRecords::<T>::deposit(&caller, T::DogeAssetId::get(), amount).unwrap();
        XGatewayRecords::<T>::withdraw(&caller, T::DogeAssetId::get(), withdrawal, b"nbGodDo7pezD2LcKN8AFMc9nMPvT1YhXcc".to_vec(), b"".to_vec().into()).unwrap();

        XGatewayRecords::<T>::withdrawal_state_insert(0, WithdrawalState::Applying);

    }: _(RawOrigin::Signed(caller), vec![0], tx_raw)
    verify {
        assert_eq!(WithdrawalProposal::<T>::get().unwrap().sig_state, VoteResult::Finish);
    }

    set_best_index {
        let best = DogeHeaderIndex {
            hash: H256::repeat_byte(1),
            height: 100,
        };
    }: _(RawOrigin::Root, best)
    verify {
        assert_eq!(Pallet::<T>::best_index(), best);
    }

    set_confirmed_index {
        let confirmed = DogeHeaderIndex {
            hash: H256::repeat_byte(1),
            height: 100,
        };
    }: _(RawOrigin::Root, confirmed)
    verify {
        assert_eq!(Pallet::<T>::confirmed_index(), Some(confirmed));
    }

    remove_pending {
        let addr = b"3AWmpzJ1kSF1cktFTDEb3qmLcdN8YydxA7".to_vec();
        let v = vec![
            DogeDepositCache {
                txid: H256::repeat_byte(1),
                balance: 100000000,
            },
            DogeDepositCache {
                txid: H256::repeat_byte(2),
                balance: 200000000,
            },
            DogeDepositCache {
                txid: H256::repeat_byte(3),
                balance: 300000000,
            },
        ];
        PendingDeposits::<T>::insert(&addr, v);
        let receiver: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Root, addr.clone(), Some(receiver))
    verify {
        assert!(Pallet::<T>::pending_deposits(&addr).is_empty());
        // assert_eq!(XAssets::<T>::usable_balance(&receiver, &T::AssetId::default()), (100000000u32 + 200000000u32 + 300000000u32).into());
    }

    remove_proposal {
        let caller = alice::<T>();
        let amount:T::Balance = 1_000_000_000u32.into();
        let withdrawal: T::Balance = 10000u32.into();

        XGatewayRecords::<T>::deposit(&caller, T::DogeAssetId::get(), amount).unwrap();
        XGatewayRecords::<T>::withdraw(&caller, T::DogeAssetId::get(), withdrawal, b"nbGodDo7pezD2LcKN8AFMc9nMPvT1YhXcc".to_vec(), b"".to_vec().into()).unwrap();
        XGatewayRecords::<T>::withdraw(&caller, T::DogeAssetId::get(), withdrawal, b"nbGodDo7pezD2LcKN8AFMc9nMPvT1YhXcc".to_vec(), b"".to_vec().into()).unwrap();

        XGatewayRecords::<T>::withdrawal_state_insert(0, WithdrawalState::Processing);
        XGatewayRecords::<T>::withdrawal_state_insert(0, WithdrawalState::Processing);

        let (tx, _, _) = withdraw_tx();
        let proposal = DogeWithdrawalProposal::<T::AccountId> {
            sig_state: VoteResult::Unfinish,
            withdrawal_id_list: vec![0, 1],
            tx,
            trustee_list: vec![],
        };

        WithdrawalProposal::<T>::put(proposal);
    }: _(RawOrigin::Root)
    verify {
        assert!(WithdrawalProposal::<T>::get().is_none());
    }

    set_btc_withdrawal_fee {
        let caller = alice::<T>();
    }: _(RawOrigin::Root,  2000000)
    verify {
    }

    set_btc_deposit_limit {
        let caller = alice::<T>();
    }: _(RawOrigin::Root,  2000000)
    verify {
    }

    set_coming_bot {
        let caller = alice::<T>();
    }: _(RawOrigin::Root,  Some(caller))
    verify {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default().build().execute_with(|| {
            assert_ok!(Pallet::<Test>::test_benchmark_push_header());
            assert_ok!(Pallet::<Test>::test_benchmark_push_transaction());
            assert_ok!(Pallet::<Test>::test_benchmark_create_dogecoin_withdraw_tx());
            assert_ok!(Pallet::<Test>::test_benchmark_set_best_index());
            assert_ok!(Pallet::<Test>::test_benchmark_set_confirmed_index());
            assert_ok!(Pallet::<Test>::test_benchmark_remove_pending());
            assert_ok!(Pallet::<Test>::test_benchmark_set_btc_withdrawal_fee());
            assert_ok!(Pallet::<Test>::test_benchmark_set_btc_deposit_limit());
            assert_ok!(Pallet::<Test>::test_benchmark_set_coming_bot());
        });
    }
}
