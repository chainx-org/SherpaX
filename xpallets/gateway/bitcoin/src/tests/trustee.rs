// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::assert_noop;
use hex_literal::hex;

use light_bitcoin::{
    chain::Transaction,
    crypto::dhash160,
    keys::{Address, AddressTypes, Network, Public, Type},
    mast::Mast,
    script::{Builder, Opcode},
};

use sherpax_primitives::{AddrStr, AssetId};
use xp_runtime::Memo;
use xpallet_gateway_common::traits::TrusteeForChain;
use xpallet_gateway_records::{WithdrawalRecordId, WithdrawalState};

use crate::trustee::create_multi_address;
use crate::{
    mock::{
        AccountId, Balance, BlockNumber, ExtBuilder, Test, XGatewayBitcoin, XGatewayBitcoinErr,
    },
    trustee::check_withdraw_tx_impl,
};
use sp_std::convert::TryInto;

#[test]
pub fn test_check_trustee_entity() {
    ExtBuilder::default().build_and_execute(|| {
        let addr_ok_3 = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        let public3 = Public::from_slice(&addr_ok_3).unwrap();
        assert_eq!(XGatewayBitcoin::check_trustee_entity(&addr_ok_3).unwrap().0, public3);
        let addr_ok_2 = hex!("0211252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        let public2 = Public::from_slice(&addr_ok_2).unwrap();
        assert_eq!(XGatewayBitcoin::check_trustee_entity(&addr_ok_2).unwrap().0, public2);

        let addr_too_long = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40cc");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_too_long), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_normal = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae4011252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_normal), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_err_type = hex!("0411252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_err_type), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_zero = hex!("020000000000000000000000000000000000000000000000000000000000000000");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_zero), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_ec_p = hex!("02fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_ec_p), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_ec_p_2 = hex!("02fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc3f");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_ec_p_2), XGatewayBitcoinErr::InvalidPublicKey);
    })
}

#[test]
pub fn test_check_withdraw_tx_impl() {
    ExtBuilder::default().build_and_execute(|| {
        let records: Vec<(
            AccountId,
            AssetId,
            Balance,
            AddrStr,
            Memo,
            BlockNumber,
            WithdrawalRecordId,
        )> = vec![
            (
                "5QQyaBsFQ5PheBsWTMZBbJG43SiaoaVfvqaPyKsean5zYvoG"
                    .parse()
                    .unwrap(),
                1,
                800_000,
                b"bc1qdnla2v5geq68gfe0wuc6vjxjxycty3d4g7zunn".to_vec(),
                b"".to_vec().into(),
                0,
                63,
            ),
            (
                "5Po4HsbvUKrA3Txf86risWBGyNUW9TdumHfwTNF8jez7QCdS"
                    .parse()
                    .unwrap(),
                1,
                19_600_000,
                b"tb1pkph70fnsuqkyyt94v42s8ztk8hp2wsru7r8nc07egx3tpavdhwus2dfdxl".to_vec(),
                b"".to_vec().into(),
                0,
                66,
            ),
            (
                "5SjBxKb3aTCMwCWqBTj97uFGcvo8n7YptQHHQ8RayumTt9h2"
                    .parse()
                    .unwrap(),
                1,
                9_000_000,
                b"tb1ps3uc28shlr64flur05xhqfjv7yu9xr9gus70ug8mvph8qpc42fcsz8v0wh".to_vec(),
                b"".to_vec().into(),
                0,
                72,
            ),
            (
                "5SShhGvZVsJyfi7ggjA6iGP7QmVsfTWgpchaMSuPyy1XjfUL"
                    .parse()
                    .unwrap(),
                1,
                8_269_999,
                b"tb1p3gvrysmm4yueffzuh6r7tynhp2ratkdsrw6y607s869wfqs52g9sl7g50q".to_vec(),
                b"".to_vec().into(),
                0,
                71,
            ),
            (
                "5Q6vBS44AD6ARmGcSLENRQxfJgNRnP5vcaKfsyRxJmzg9A3e"
                    .parse()
                    .unwrap(),
                1,
                10_000_000,
                b"tb1pchw4p50qyj972k6kzle866ww2rn4twfhdf2wrwxw58dycf5eexusk5ekqw".to_vec(),
                b"".to_vec().into(),
                0,
                70,
            ),
            (
                "5UVrHLKqXPb3jMKGaggdxsDzbhd4cdGqB9A779txjut4trJN"
                    .parse()
                    .unwrap(),
                1,
                999_999,
                b"tb1pgu80wj6hyqw55u3eylkzh9dwe4fuunxh25k824dq64083k3sqqrsd5qaqz".to_vec(),
                b"".to_vec().into(),
                0,
                67,
            ),
            (
                "5TSzjbXSCcHHL8FUDyaPaquV32RwXRkw8WTgunQzNMEkooEE"
                    .parse()
                    .unwrap(),
                1,
                1_000_000,
                b"tb1pxawgx63ylyex9wn3ux3dcazwf42cx5nk3hln3s9zw6562dpwy3fqzaxutk".to_vec(),
                b"".to_vec().into(),
                0,
                73,
            ),
            (
                "5TGpkEDWAv3fZM8W97FqgURtopo7X3Np14rJ356WNEgRK2q5"
                    .parse()
                    .unwrap(),
                1,
                1_800_000,
                b"tb1parxya8w5ks04hz8v0h89nm38m32y2zc882ue7ul2wvlqd888en8shfzt2q".to_vec(),
                b"".to_vec().into(),
                0,
                68,
            ),
            (
                "5Qn3rMd52Rjt7sEnmBfVjjRbBFUFijyMUDKyRZhp1Xr7rryQ"
                    .parse()
                    .unwrap(),
                1,
                1_000_000,
                b"tb1p9le6mqe4gkcq5vhcyg0xj2srwf76tdamhd3uhfwezkhur8pfsh5qe6dkfd".to_vec(),
                b"".to_vec().into(),
                0,
                69,
            ),
        ];
        records.iter().for_each(|record| {
            let r = xpallet_gateway_records::WithdrawalRecordOf::<Test>::new(
                record.0.clone(),
                record.1,
                record.2,
                record.3.clone(),
                record.4.clone(),
                record.5,
            );
            xpallet_gateway_records::PendingWithdrawals::<Test>::insert(record.6, r);
            xpallet_gateway_records::WithdrawalStateOf::<Test>::insert(
                record.6,
                WithdrawalState::Applying,
            );
        });
        let withdrawal_id_list = vec![63, 66, 67, 68, 69, 70, 71, 72, 73];
        let tx: Transaction = "02000000000102b1712f279a048f7e7af6aeaf02a6af412669937dc7c7abe7c0d40b305fa619ab0b00000000000000005f20fc7ac15075f7921901fc71c2e290d869b58ba6fa0d1351205d007123af410000000000000000000a20d6130000000000225120e8cc4e9dd4b41f5b88ec7dce59ee27dc54450b073ab99f73ea733e069ce7cccf20a1070000000000225120375c836a24f93262ba71e1a2dc744e4d558352768dff38c0a276a9a5342e2452e0930400000000001600146cffd53288c83474272f7731a648d23130b245b51fa1070000000000225120470ef74b57201d4a723927ec2b95aecd53ce4cd7552c7555a0d55e78da30000720b38100000000002251208479851e17f8f554ff837d0d70264cf138530ca8e43cfe20fb606e70071552716071230100000000225120b06fe7a670e02c422cb565550389763dc2a7407cf0cf3c3fd941a2b0f58dbbb920a10700000000002251202ff3ad833545b00a32f8221e692a03727da5b7bbbb63cba5d915afc19c2985e860f5900000000000225120c5dd50d1e0248be55b5617f27d69ce50e755b9376a54e1b8cea1da4c2699c9b98f8f7600000000002251208a1832437ba93994a45cbe87e592770a87d5d9b01bb44d3fd03e8ae48214520bac4dd6dd01000000225120cb72accaf99f5243ba473b14bfeb7372f204315ca0b9dcd7475565fc5a639d7503405595326a3c65c9c7db56a09fc31124d1f49c42f3968c2d576e776961e7d7cdfaf0254a1b59927535c9fe88d492ac1912ed9523c9b8bf2eaa990fb133f8c3539222202c227e2f76c4c496b03a15c96beb8a44787da876a551b9606843c05c08cb6001acfd0101c01b62c6adc3fdb99fce4e7771e3fd93dbefa66b4413e301fcb0c7e1db69f99f11657f74b924d815ac3575a7beacaf4b1ed4d4b9344b04963a25dc97a322a46093f945d9e21c0605b7e415a5753ec01cf64d8f936586bcd61e2d124e6ef686fb7f4800edfeb27eff3b91e310ba9d1a04ff0e1b12567abceeba5633cb1c18b9aab9ee29678aef889e17ca9f3ee96ede548692d2e5512ab5cb249322d081ce46184ad4354cdd45896c49c245ed41120439f2008701f16e9bd1fcf79d1d5f302e90ad97b8e06b83be8a3c64410aa0670aade56ad65c54af50701dc100bdd96417fb6e868bc11ca10d6d482299f397d4fbfbf24916b795730672a351c7481f4aa494330340ec736273ea0d0d65f88c7adadde84ac2db1b7d22ca55a3a93280de865d0c40bc58c517206532dadde7b8e95e9a43a5588c4d9444dd92df29defccc04fe729f6422202c227e2f76c4c496b03a15c96beb8a44787da876a551b9606843c05c08cb6001acfd0101c01b62c6adc3fdb99fce4e7771e3fd93dbefa66b4413e301fcb0c7e1db69f99f11657f74b924d815ac3575a7beacaf4b1ed4d4b9344b04963a25dc97a322a46093f945d9e21c0605b7e415a5753ec01cf64d8f936586bcd61e2d124e6ef686fb7f4800edfeb27eff3b91e310ba9d1a04ff0e1b12567abceeba5633cb1c18b9aab9ee29678aef889e17ca9f3ee96ede548692d2e5512ab5cb249322d081ce46184ad4354cdd45896c49c245ed41120439f2008701f16e9bd1fcf79d1d5f302e90ad97b8e06b83be8a3c64410aa0670aade56ad65c54af50701dc100bdd96417fb6e868bc11ca10d6d482299f397d4fbfbf24916b795730672a351c7481f4aa4943300000000".parse().unwrap();
        check_withdraw_tx_impl::<Test>(&tx, &withdrawal_id_list);
    })
}

#[test]
pub fn test_multi_address() {
    let pubkey1_bytes = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
    let pubkey2_bytes = hex!("02e34d10113f2dd162e8d8614a4afbb8e2eb14eddf4036042b35d12cf5529056a2");
    let pubkey3_bytes = hex!("023e505c48a955e759ce61145dc4a9a7447425290b8483f4e36f05169e7967c86d");

    let script = Builder::default()
        .push_opcode(Opcode::OP_2)
        .push_bytes(&pubkey1_bytes)
        .push_bytes(&pubkey2_bytes)
        .push_bytes(&pubkey3_bytes)
        .push_opcode(Opcode::OP_3)
        .push_opcode(Opcode::OP_CHECKMULTISIG)
        .into_script();
    let multisig_address = Address {
        kind: Type::P2SH,
        network: Network::Testnet,
        hash: AddressTypes::Legacy(dhash160(&script)),
    };
    assert_eq!(
        "2MtAUgQmdobnz2mu8zRXGSTwUv9csWcNwLU",
        multisig_address.to_string()
    );
}

#[test]
fn test_create_multi_address() {
    let mut hot_keys = Vec::new();
    let pubkey1_bytes = hex!("03f72c448a0e59f48d4adef86cba7b278214cece8e56ef32ba1d179e0a8129bdba");
    let pubkey2_bytes = hex!("0306117a360e5dbe10e1938a047949c25a86c0b0e08a0a7c1e611b97de6b2917dd");
    let pubkey3_bytes = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
    let pubkey4_bytes = hex!("0227e54b65612152485a812b8856e92f41f64788858466cc4d8df674939a5538c3");
    hot_keys.push(Public::from_slice(&pubkey1_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey2_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey3_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey4_bytes).unwrap());

    let mut cold_keys = Vec::new();
    let pubkey5_bytes = hex!("02a79800dfed17ad4c78c52797aa3449925692bc8c83de469421080f42d27790ee");
    let pubkey6_bytes = hex!("03ece1a20b5468b12fd7beda3e62ef6b2f6ad9774489e9aff1c8bc684d87d70780");
    let pubkey7_bytes = hex!("02e34d10113f2dd162e8d8614a4afbb8e2eb14eddf4036042b35d12cf5529056a2");
    let pubkey8_bytes = hex!("020699bf931859cafdacd8ac4d3e055eae7551427487e281e3efba618bdd395f2f");
    cold_keys.push(Public::from_slice(&pubkey5_bytes).unwrap());
    cold_keys.push(Public::from_slice(&pubkey6_bytes).unwrap());
    cold_keys.push(Public::from_slice(&pubkey7_bytes).unwrap());
    cold_keys.push(Public::from_slice(&pubkey8_bytes).unwrap());

    ExtBuilder::default().build_and_execute(|| {
        let hot_info = create_multi_address::<Test>(&hot_keys, 3).unwrap();
        let cold_info = create_multi_address::<Test>(&cold_keys, 3).unwrap();
        let real_hot_addr = b"2N1CPZyyoKj1wFz2Fy4gEHpSCVxx44GtyoY".to_vec();
        let real_cold_addr = b"2N24ytjE3MtkMpYWo8LrTfnkbpyaJGyQbCA".to_vec();
        assert_eq!(hot_info.addr, real_hot_addr);
        assert_eq!(cold_info.addr, real_cold_addr);

        let pks = [
            169, 20, 87, 55, 193, 151, 147, 67, 146, 12, 238, 164, 14, 124, 125, 104, 178, 100,
            176, 239, 250, 62, 135,
        ];
        let addr: Address = String::from_utf8_lossy(&hot_info.addr).parse().unwrap();
        let pk = match addr.hash {
            AddressTypes::Legacy(h) => h.as_bytes().to_vec(),
            AddressTypes::WitnessV0ScriptHash(_) => todo!(),
            AddressTypes::WitnessV0KeyHash(_) => todo!(),
            AddressTypes::WitnessV1Taproot(_) => todo!(),
        };
        let mut pubkeys = vec![Opcode::OP_HASH160 as u8, Opcode::OP_PUSHBYTES_20 as u8];
        pubkeys.extend_from_slice(&pk);
        pubkeys.push(Opcode::OP_EQUAL as u8);
        assert_eq!(pubkeys, pks);
    });
}
#[test]
fn test_create_taproot_address() {
    let mut hot_keys = Vec::new();
    let pubkey1_bytes = hex!("0283f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2");
    let pubkey2_bytes = hex!("027a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d");
    let pubkey3_bytes = hex!("02c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f");
    let pubkey4_bytes = hex!("0237322a5008a1b26ac72778167e770e1fa2272cfd9f9fe0f2c20bd41fe051da6c");
    let pubkey5_bytes = hex!("03227368d7168173229f1898b8448dc5c0640ce35eb574639e42ec515b73d5f2d3");
    let pubkey6_bytes = hex!("027196048a63ec7a3b9cb5a23a51952503e7fca8de2ec42388952e067d39fc83ff");
    let pubkey7_bytes = hex!("025b9cd3170511ced44caf8067b1a759dfd7b2f2d52352c4a788b15a1cbc3afa56");
    let pubkey8_bytes = hex!("030709034ebd0964796a11fab08fce940524bd7dfdbd99b8a7a7618b71754e7584");
    let pubkey9_bytes = hex!("02f8c11e20a30a0683539ea579725eced3e055a4dcbef88f69162805f3dc609760");
    let pubkey10_bytes = hex!("0379f12ca4c0fb587616aba27ab7f66245120b4e1f83a413769a5779af48146e87");
    hot_keys.push(Public::from_slice(&pubkey1_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey2_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey3_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey4_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey5_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey6_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey7_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey8_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey9_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey10_bytes).unwrap());
    ExtBuilder::default().build_and_execute(|| {
        let pks = hot_keys
            .into_iter()
            .map(|k| k.try_into().unwrap())
            .collect::<Vec<_>>();
        let mast = Mast::new(pks, 7_u32).unwrap();
        let threshold_addr: Address = mast
            .generate_address(&crate::Pallet::<Test>::network_id().to_string())
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(mast.pubkeys.len(), 120);
        assert_eq!(
            threshold_addr.to_string(),
            "tb1psaktm6w6nrh5xs8umla9qaw6zjarr4yuqk3m4x8pzc6ekve93v7ss20kuq"
        )
    })
}
