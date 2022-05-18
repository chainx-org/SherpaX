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

fn generate_blocks_3836100_3836160() -> BTreeMap<u32, BlockHeader> {
    let bytes = include_bytes!("./res/headers-3836100-3836160.raw");
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
    // block height: 3836144
    const RAW_TX: &str = "01000000019651fae85e1ce20ee7e1d934a548802698384d3bf8cda619cbf10a916bb2374e00000000fdfd000047304402202f082c4ce318bbb52c41d88e71318ddee0771a6ca2d0eb5e50e1297cf1edbe9d022052d4e21919ff6b8281ade9383cb5d69ad5b4a76b0f0a5471c8335cd2b3114d460148304502210090731991cffa14f0ec2ad1ceaeedc8eb4204294e4c8b61896aa9159a41f2b8f3022054733e7a8915ed730d433052594f9699f043ee1935813fc42da85f121ece46b9014c695221032f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85210251e0dc3d9709d860c49785fc84b62909d991cffd81592f6994c452438f91b6a22102a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f853ae000000000280969800000000001976a9144da9bb5dea4c42219a2a120523d1a0ce6c268f3788ac00b4c4040000000017a9140473e14aec27f8edb5baa7ac03a600b0946517518700000000";
    let tx = RAW_TX.parse::<Transaction>().unwrap();

    const RAW_TX_PREV: &str = "010000000144b4ae11c340569056655af8b875a9d6af881b599dc0fa7fa3dff59d6ade0bce020000008a47304402207ea6837fea50ea3f84aa3100ff1c48448d4610eb71c7fc9adb7cc3d9dba89d36022063ee329c6b4ee4c9e1a7deb76b1795132a2de73da55e75cab0b2a0dfeb9fb6740141042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3000000000300e1f5050000000017a9140473e14aec27f8edb5baa7ac03a600b094651751870000000000000000326a303555543838746b4675457668506367577178486f686b584844684c6b3954666b704d595455684748533654683834384700bc522a020000001976a9144afe03f863d27be1cfb7ec0859c4ff89569bb23988ac00000000";
    let prev_tx = RAW_TX_PREV.parse::<Transaction>().unwrap();

    const RAW_PROOF: &str = "0200000002de75689cf7aff62afba4a3d10f5cef2b134531d487707ae5ccc8d7f84e15e9adee68b66ddf9fbaaff222b101e1e97fb70595611bac081d8a8229e292f5264c1a0105";
    let proof = hex::decode(RAW_PROOF).unwrap();
    let merkle_proof: PartialMerkleTree = serialization::deserialize(Reader::new(&proof)).unwrap();

    let header = generate_blocks_3836100_3836160()[&3836144];
    let info = DogeRelayedTxInfo {
        block_hash: header.hash(),
        merkle_proof,
    };
    (tx, info.encode(), prev_tx)
}

// push header 3836100 - 3782230
fn prepare_headers<T: Config>(caller: &T::AccountId) {
    for (height, header) in generate_blocks_3836100_3836160() {
        if height == 3836100 {
            continue;
        }
        let header = serialization::serialize(&header).into();
        Pallet::<T>::push_header(RawOrigin::Signed(caller.clone()).into(), header).unwrap();
    }
}

benchmarks! {
    push_header {
        let receiver: T::AccountId = whitelisted_caller();
        let insert_height = 3836100 + 1;
        let header = generate_blocks_3836100_3836160()[&insert_height];
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

        let withdrawal: T::Balance = 10_000_000u32.into();

        // withdrawal + runtime-benchmark's withdrawal fee
        #[cfg(feature = "runtime-benchmarks")]
        let withdrawal: T::Balance = 1_500_000u32.into();

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
        // assert_eq!(XAssets::<T>::usable_balance(&receiver, &T::DogeAssetId::get()), (100000000u32 + 200000000u32 + 300000000u32).into());
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

    set_doge_withdrawal_fee {
        let caller = alice::<T>();
    }: _(RawOrigin::Root,  2000000)
    verify {
    }

    set_doge_deposit_limit {
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
            assert_ok!(Pallet::<Test>::test_benchmark_set_doge_withdrawal_fee());
            assert_ok!(Pallet::<Test>::test_benchmark_set_doge_deposit_limit());
            assert_ok!(Pallet::<Test>::test_benchmark_set_coming_bot());
        });
    }
}
