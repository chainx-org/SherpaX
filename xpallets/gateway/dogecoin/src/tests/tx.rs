// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

#![allow(non_upper_case_globals)]

use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use sp_core::crypto::{set_default_ss58_version, Ss58AddressFormatRegistry};

use light_bitcoin::{
    chain::Transaction,
    keys::{Address, Network},
    merkle::PartialMerkleTree,
    serialization::{self, Reader},
};

use xp_gateway_dogecoin::{AccountExtractor, DogeTxMetaType, DogeTxTypeDetector};

use crate::{
    mock::*,
    tx::process_tx,
    types::{DogeRelayedTxInfo, DogeTxResult, DogeTxState, DogeWithdrawalProposal, VoteResult},
    Config, WithdrawalProposal,
};

// Tyoe is p2tr. Address farmat is Mainnet.:
const DEPOSIT_HOT_ADDR: &str = "2Mw36zb6tAdZ6vjPvS3fXvR1r1wg1K8UXX7";
// Tyoe is p2sh. Address farmat is Mainnet.
const DEPOSIT_COLD_ADDR: &str = "3Ac85hjgeyNX96Q4BqUoAH5bh6gARxRDJm";

lazy_static::lazy_static! {
    // deposit without op return, output addr is DEPOSIT_HOT_ADDR. Withdraw is an example of spending from the script path.
    static ref deposit_input_account: Vec<u8> = b"nb2gZmphdD5fEVLDHdYAKp5Lpb1w4p5R2k".to_vec();
    // https://blockexplorer.one/dogecoin/testnet/tx/c4cefe8a1b70e8e49b51dcf5b895651f3e5622acde917772cd573a099446fb43
    static ref deposit_prev: Transaction = "0100000001802ca50a6c12f3b6e1da2dad8c0d72ccfd894bf410175217ec955fca832dc668010000008b483045022100da02ce077c88c8d955c7d53f9446c6c1675da1fcfa3b7d94640af44ef53aa9ee02202e9a7eeb93a5eb67342136b25d756bc175b7285105b1dbf5f8c336759e13ebd80141042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3000000000200e1f505000000001976a9144da9bb5dea4c42219a2a120523d1a0ce6c268f3788ac0000000000000000326a3035516a706f3772516e7751657479736167477a6334526a376f737758534c6d4d7141754332416255364c464646476a3800000000".parse().unwrap();
    // https://blockexplorer.one/dogecoin/testnet/tx/e87356328d8369b9ac9752076d1f3807613b6a1447639b0e85be9b418d802699
    static ref deposit: Transaction = "010000000143fb4694093a57cd727791deac22563e1f6595b8f5dc519be4e8701b8afecec4000000008a47304402205ef330d36268379c78e32cfc3b04b3bfc8d595c9c161b65a9e81f866331dbdee02206c0e960eeeb74ea02deac4328251f5a62b39b185aa5a451134b77e873619f123014104a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b80000000002809698000000000017a9142995ac346d93b015e2941715d432af5ac4e1010c870000000000000000326a3035516a706f3772516e7751657479736167477a6334526a376f737758534c6d4d7141754332416255364c464646476a3800000000".parse().unwrap();
    static ref withdraw_prev: Transaction = "010000000143fb4694093a57cd727791deac22563e1f6595b8f5dc519be4e8701b8afecec4000000008a47304402205ef330d36268379c78e32cfc3b04b3bfc8d595c9c161b65a9e81f866331dbdee02206c0e960eeeb74ea02deac4328251f5a62b39b185aa5a451134b77e873619f123014104a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b80000000002809698000000000017a9142995ac346d93b015e2941715d432af5ac4e1010c870000000000000000326a3035516a706f3772516e7751657479736167477a6334526a376f737758534c6d4d7141754332416255364c464646476a3800000000".parse().unwrap();
    // https://blockexplorer.one/dogecoin/testnet/tx/55728d2dc062a9dfe21bae44e87665b270382c8357f14b2a1a4b2b9af92a894a
    static ref withdraw: Transaction = "01000000019926808d419bbe850e9b6347146a3b6107381f6d075297acb969838d325673e800000000fd5c010047304402205a546a45118fdfeff5abb6470cfcda5ce8927227ff5aabacae2a3dda9a46ff900220574ca3ef630b6eb15e0e4744c0f4804452406a22e61639f5c320de78ba400b600147304402206a575b863f66dc69bc2104f117c354b18cdd8f612edb582a506a5b141fa2a74e02204318df9774970f0cbef1224821fa7dfecbc62949a4db48c3d8f0ee9eeafef448014cc95241042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3410451e0dc3d9709d860c49785fc84b62909d991cffd81592f6994c452438f91b6a2e586541c4b3bc1ebeb5fb9fad2ed2e696b2175c54458ab6f103717cbeeb4e52c4104a09e8182977710bab64472c0ecaf9e52255a890554a00a62facd05c0b13817f8995bf590851c19914bfc939d53365b90cc2f0fcfddaca184f0c1e7ce1736f0b853ae000000000240420f00000000001976a9144da9bb5dea4c42219a2a120523d1a0ce6c268f3788ac00127a000000000017a9142995ac346d93b015e2941715d432af5ac4e1010c8700000000".parse().unwrap();

    // deposit with op return, output addr is DEPOSIT_HOT_ADDR. Withdraw is an example of spending from the script path.
    static ref op_account: AccountId = "5Qjpo7rQnwQetysagGzc4Rj7oswXSLmMqAuC2AbU6LFFFGj8".parse().unwrap();
    // https://signet.bitcoinexplorer.org/tx/1f8e0f7dfa37b184244d022cdf2bc7b8e0bac8b52143ea786fa3f7bbe049eeae#JSON
    // static ref deposit_taproot2_prev: Transaction = "020000000001014be640313b023c3c731b7e89c3f97bebcebf9772ea2f7747e5604f4483a447b601000000000000000002a0860100000000002251209a9ea267884f5549c206b2aec2bd56d98730f90532ea7f7154d4d4f923b7e3bbc027090000000000225120c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f01404dc68b31efc1468f84db7e9716a84c19bbc53c2d252fd1d72fa6469e860a74486b0990332b69718dbcb5acad9d48634d23ee9c215ab15fb16f4732bed1770fdf00000000".parse().unwrap();
    // https://signet.bitcoinexplorer.org/tx/8e5d37c768acc4f3e794a10ad27bf0256237c80c22fa67117e3e3e1aec22ea5f#JSON
    // static ref deposit_taproot2: Transaction = "02000000000101aeee49e0bbf7a36f78ea4321b5c8bae0b8c72bdf2c024d2484b137fa7d0f8e1f01000000000000000003a0860100000000002251209a9ea267884f5549c206b2aec2bd56d98730f90532ea7f7154d4d4f923b7e3bb0000000000000000326a3035516a706f3772516e7751657479736167477a6334526a376f737758534c6d4d7141754332416255364c464646476a38801a060000000000225120c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f01409e325889515ed47099fdd7098e6fafdc880b21456d3f368457de923f4229286e34cef68816348a0581ae5885ede248a35ac4b09da61a7b9b90f34c200872d2e300000000".parse().unwrap();
    // static ref withdraw_taproot2_prev: Transaction = "02000000000101aeee49e0bbf7a36f78ea4321b5c8bae0b8c72bdf2c024d2484b137fa7d0f8e1f01000000000000000003a0860100000000002251209a9ea267884f5549c206b2aec2bd56d98730f90532ea7f7154d4d4f923b7e3bb0000000000000000326a3035516a706f3772516e7751657479736167477a6334526a376f737758534c6d4d7141754332416255364c464646476a38801a060000000000225120c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f01409e325889515ed47099fdd7098e6fafdc880b21456d3f368457de923f4229286e34cef68816348a0581ae5885ede248a35ac4b09da61a7b9b90f34c200872d2e300000000".parse().unwrap();
    // https://signet.bitcoinexplorer.org/tx/0f592933b493bedab209851cb2cf07871558ff57d86d645877b16651479b51a2#JSON
    // static ref withdraw_taproot2: Transaction = "020000000001015fea22ec1a3e3e7e1167fa220cc8376225f07bd20aa194e7f3c4ac68c7375d8e0000000000000000000250c3000000000000225120c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f409c0000000000002251209a9ea267884f5549c206b2aec2bd56d98730f90532ea7f7154d4d4f923b7e3bb03402639d4d9882f6e7e42db38dbd2845c87b131737bf557643ef575c49f8fc6928869d9edf5fd61606fb07cced365fdc2c7b637e6ecc85b29906c16d314e7543e94222086a60c7d5dd3f4931cc8ad77a614402bdb591c042347c89281c48c7e9439be9dac61c0e56a1792f348690cdeebe60e3db6c4e94d94e742c619f7278e52f6cbadf5efe96a528ba3f61a5b0d4fbceea425a9028381458b32492bccc3f1faa473a649e23605554f5ea4b4044229173719228a35635eeffbd8a8fe526270b737ad523b99f600000000".parse().unwrap();

    // Convert between DEPOSIT_HOT_ADDR and DEPOSIT_COLD_ADDR
    // https://signet.bitcoinexplorer.org/tx/0f592933b493bedab209851cb2cf07871558ff57d86d645877b16651479b51a2#JSON
    // static ref hot_to_cold_prev: Transaction = "020000000001015fea22ec1a3e3e7e1167fa220cc8376225f07bd20aa194e7f3c4ac68c7375d8e0000000000000000000250c3000000000000225120c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f409c0000000000002251209a9ea267884f5549c206b2aec2bd56d98730f90532ea7f7154d4d4f923b7e3bb03402639d4d9882f6e7e42db38dbd2845c87b131737bf557643ef575c49f8fc6928869d9edf5fd61606fb07cced365fdc2c7b637e6ecc85b29906c16d314e7543e94222086a60c7d5dd3f4931cc8ad77a614402bdb591c042347c89281c48c7e9439be9dac61c0e56a1792f348690cdeebe60e3db6c4e94d94e742c619f7278e52f6cbadf5efe96a528ba3f61a5b0d4fbceea425a9028381458b32492bccc3f1faa473a649e23605554f5ea4b4044229173719228a35635eeffbd8a8fe526270b737ad523b99f600000000".parse().unwrap();
    // https://signet.bitcoinexplorer.org/tx/917a751b9ccd91c7e184b028739a5520420df5cf04cd851a6ddf51f7bf33cf8a#JSON
    // static ref hot_to_cold: Transaction = "02000000000101a2519b475166b17758646dd857ff58158707cfb21c8509b2dabe93b43329590f01000000000000000002204e00000000000017a91461cc314f71a88ebb492939784ca2663afaa8e88c8710270000000000002251209a9ea267884f5549c206b2aec2bd56d98730f90532ea7f7154d4d4f923b7e3bb0340aba2ce052b2fce8285ad550c4fd9182c8c8b4d2bcb91a4fd548d41c4a52f1137910ec79bf64ebc908db5c2713908e4cbb63d4e57dd723fdaf90f281b091d6f3e222086a60c7d5dd3f4931cc8ad77a614402bdb591c042347c89281c48c7e9439be9dac61c0e56a1792f348690cdeebe60e3db6c4e94d94e742c619f7278e52f6cbadf5efe96a528ba3f61a5b0d4fbceea425a9028381458b32492bccc3f1faa473a649e23605554f5ea4b4044229173719228a35635eeffbd8a8fe526270b737ad523b99f600000000".parse().unwrap();
    // Todo generate cold to hot
    // static ref cold_to_hot_prev: Transaction = "01000000015dfd7ae51ea70f3dfc9d4a49d57ae0d02660f089204fc8c4d086624d065f85620000000000000000000180010b270100000017a91495a12f1eba77d085711e9c837d04e4d8868a83438700000000".parse().unwrap();
    // static ref cold_to_hot: Transaction = "0100000001bc7be600cba239950fd664995bb9bc2cb88a29d95ddd49625644ef188c98012e0000000000000000000180010b270100000022512052898a03a9f04bb83f8a48fb953089de10e6ee70658b059551ebf7c008b05b7a00000000".parse().unwrap();
}

fn mock_detect_transaction_type<T: Config>(
    tx: &Transaction,
    prev_tx: Option<&Transaction>,
) -> DogeTxMetaType<T::AccountId> {
    let btc_tx_detector = DogeTxTypeDetector::new(Network::Mainnet, 0);
    let current_trustee_pair = (
        DEPOSIT_HOT_ADDR.parse::<Address>().unwrap(),
        DEPOSIT_COLD_ADDR.parse::<Address>().unwrap(),
    );
    btc_tx_detector.detect_transaction_type::<T::AccountId, _>(
        tx,
        prev_tx,
        |script| T::AccountExtractor::extract_account(script),
        current_trustee_pair,
        None,
    )
}

#[test]
fn test_detect_tx_type() {
    set_default_ss58_version(Ss58AddressFormatRegistry::ChainxAccount.into());
    match mock_detect_transaction_type::<Test>(&deposit, None) {
        DogeTxMetaType::Deposit(info) => {
            assert!(info.input_addr.is_none() && info.op_return.is_some())
        }
        _ => unreachable!("wrong type"),
    }
    // match mock_detect_transaction_type::<Test>(&deposit_taproot2, None) {
    //     DogeTxMetaType::Deposit(info) => {
    //         assert!(info.input_addr.is_none() && info.op_return.is_some())
    //     }
    //     _ => unreachable!("wrong type"),
    // }

    match mock_detect_transaction_type::<Test>(&deposit, Some(&deposit_prev)) {
        DogeTxMetaType::Deposit(info) => {
            assert!(info.input_addr.is_some() && info.op_return.is_some())
        }
        _ => unreachable!("wrong type"),
    }

    // match mock_detect_transaction_type::<Test>(&deposit_taproot2, Some(&deposit_taproot2_prev)) {
    //     DogeTxMetaType::Deposit(info) => {
    //         assert!(info.input_addr.is_some() && info.op_return.is_some())
    //     }
    //     _ => unreachable!("wrong type"),
    // }

    match mock_detect_transaction_type::<Test>(&withdraw, Some(&withdraw_prev)) {
        DogeTxMetaType::Withdrawal => {}
        _ => unreachable!("wrong type"),
    }

    // match mock_detect_transaction_type::<Test>(&withdraw_taproot2, Some(&withdraw_taproot2_prev)) {
    //     DogeTxMetaType::Withdrawal => {}
    //     _ => unreachable!("wrong type"),
    // }

    // // hot_to_cold
    // // if not pass a prev, would judge to a deposit, but this deposit could not be handled due to
    // // opreturn and input_addr are all none, or if all send to cold, it would be Irrelevance
    // match mock_detect_transaction_type::<Test>(&hot_to_cold, None) {
    //     DogeTxMetaType::Deposit(info) => {
    //         assert!(info.input_addr.is_none() && info.op_return.is_none())
    //     }
    //     _ => unreachable!("wrong type"),
    // }
    // // then if provide prev, it would be judge to a HotAndCold
    // match mock_detect_transaction_type::<Test>(&hot_to_cold, Some(&hot_to_cold_prev)) {
    //     DogeTxMetaType::HotAndCold => {}
    //     _ => unreachable!("wrong type"),
    // }

    // // cold_to_hot
    // // if not pass a prev, would judge to a deposit, but this deposit could not be handled due to
    // // opreturn and input_addr are all none
    // match mock_detect_transaction_type::<Test>(&cold_to_hot, None) {
    //     DogeTxMetaType::Deposit(info) => {
    //         assert!(info.input_addr.is_none() && info.op_return.is_none())
    //     }
    //     _ => unreachable!("wrong type"),
    // }
    // // then if provide prev, it would be judge to a HotAndCold
    // match mock_detect_transaction_type::<Test>(&cold_to_hot, Some(&cold_to_hot_prev)) {
    //     DogeTxMetaType::HotAndCold => {}
    //     _ => unreachable!("wrong type"),
    // }
}

fn mock_process_tx<T: Config>(tx: Transaction, prev_tx: Option<Transaction>) -> DogeTxState {
    let network = Network::DogeCoinTestnet;
    let min_deposit = 0;
    let current_trustee_pair = (
        DEPOSIT_HOT_ADDR.parse::<Address>().unwrap(),
        DEPOSIT_COLD_ADDR.parse::<Address>().unwrap(),
    );
    let previous_trustee_pair = None;
    process_tx::<T>(
        tx,
        prev_tx,
        network,
        min_deposit,
        current_trustee_pair,
        previous_trustee_pair,
    )
}

#[test]
fn test_process_tx() {
    set_default_ss58_version(Ss58AddressFormatRegistry::ChainxAccount.into());
    ExtBuilder::default().build_and_execute(|| {
        // with op return and input address
        let r = mock_process_tx::<Test>(deposit.clone(), None);
        assert_eq!(r.result, DogeTxResult::Success);

        // with op return and input address
        let r = mock_process_tx::<Test>(deposit.clone(), Some(deposit_prev.clone()));
        assert_eq!(r.result, DogeTxResult::Success);

        // withdraw
        WithdrawalProposal::<Test>::put(DogeWithdrawalProposal {
            sig_state: VoteResult::Unfinish,
            withdrawal_id_list: vec![],
            tx: withdraw.clone(),
            trustee_list: vec![],
        });

        let r = mock_process_tx::<Test>(withdraw.clone(), None);
        assert_eq!(r.result, DogeTxResult::Failure);
        let r = mock_process_tx::<Test>(withdraw.clone(), Some(withdraw_prev.clone()));
        assert_eq!(r.result, DogeTxResult::Success);

        // with op return and without input address
        // let r = mock_process_tx::<Test>(deposit_taproot2.clone(), None);
        // assert_eq!(r.result, DogeTxResult::Success);
        // // assert_eq!(Assets::balance(X_BTC, op_account), 100000);
        // assert_eq!(XGatewayCommon::bound_addrs(&op_account), Default::default());
        // // with op return and input address
        // let r = mock_process_tx::<Test>(
        //     deposit_taproot2.clone(),
        //     Some(deposit_taproot2_prev.clone()),
        // );
        // assert_eq!(r.result, DogeTxResult::Success);
        // assert_eq!(XAssets::usable_balance(&op_account, &X_BTC), 300000);

        // withdraw
        // WithdrawalProposal::<Test>::put(DogeWithdrawalProposal {
        //     sig_state: VoteResult::Unfinish,
        //     withdrawal_id_list: vec![],
        //     tx: withdraw_taproot2.clone(),
        //     trustee_list: vec![],
        // });

        // let r = mock_process_tx::<Test>(withdraw_taproot2.clone(), None);
        // assert_eq!(r.result, DogeTxResult::Failure);
        // let r = mock_process_tx::<Test>(
        //     withdraw_taproot2.clone(),
        //     Some(withdraw_taproot2_prev.clone()),
        // );
        // assert_eq!(r.result, DogeTxResult::Success);

        // hot and cold
        // let r = mock_process_tx::<Test>(hot_to_cold.clone(), None);
        // assert_eq!(r.result, DogeTxResult::Failure);
        // let r = mock_process_tx::<Test>(hot_to_cold.clone(), Some(hot_to_cold_prev.clone()));
        // assert_eq!(r.tx_type, DogeTxType::HotAndCold);
        // assert_eq!(r.result, DogeTxResult::Success);
    })
}

#[test]
fn test_push_tx_call() {
    set_default_ss58_version(Ss58AddressFormatRegistry::ChainxAccount.into());
    // https://blockchain.info/rawtx/f1a9161a045a01db7ae02b8c0531e2fe2e9740efe30afe6d84a12e3cac251344?format=hex
    let normal_deposit: Transaction = "010000000144b4ae11c340569056655af8b875a9d6af881b599dc0fa7fa3dff59d6ade0bce020000008a47304402207ea6837fea50ea3f84aa3100ff1c48448d4610eb71c7fc9adb7cc3d9dba89d36022063ee329c6b4ee4c9e1a7deb76b1795132a2de73da55e75cab0b2a0dfeb9fb6740141042f7e2f0f3e912bf416234913b388393beb5092418fea986e45c0b9633adefd85168f3b1d13ae29651c29e424760b3795fc78152ac119e0dc4e2b9055329099b3000000000300e1f5050000000017a9140473e14aec27f8edb5baa7ac03a600b094651751870000000000000000326a303555543838746b4675457668506367577178486f686b584844684c6b3954666b704d595455684748533654683834384700bc522a020000001976a9144afe03f863d27be1cfb7ec0859c4ff89569bb23988ac00000000".parse().unwrap();
    let tx = serialization::serialize(&normal_deposit);
    let headers = generate_blocks_3836100_3836160();
    let block_hash = headers[&3836138].hash();

    let raw_proof = hex::decode("0200000002ed5df33dc0bcb73dbd6adacdb5a7cb71377f62c4b15320cf59735a29bee2becc9651fae85e1ce20ee7e1d934a548802698384d3bf8cda619cbf10a916bb2374e0105").unwrap();
    let proof: PartialMerkleTree = serialization::deserialize(Reader::new(&raw_proof)).unwrap();

    ExtBuilder::default().build_and_execute(|| {
        let confirmed = XGatewayDogecoin::confirmation_number();
        // insert headers
        for i in 3836101..=3836154 + confirmed {
            assert_ok!(XGatewayDogecoin::apply_push_header(headers[&i]));
        }
        let info = DogeRelayedTxInfo {
            block_hash,
            merkle_proof: proof,
        }
        .encode();

        assert_ok!(XGatewayDogecoin::push_transaction(
            frame_system::RawOrigin::Signed(alice()).into(),
            tx.clone().into(),
            info.clone(),
            None,
        ));

        // reject replay
        assert_noop!(
            XGatewayDogecoin::push_transaction(
                frame_system::RawOrigin::Signed(alice()).into(),
                tx.clone().into(),
                info,
                None,
            ),
            XGatewayDogecoinErr::ReplayedTx,
        );
    });
}
