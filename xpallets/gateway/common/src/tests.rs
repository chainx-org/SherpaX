// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use crate::{
    mock::{bob, charlie, dave, Assets, ExtBuilder, Test, XGatewayCommon, XGatewayRecords},
    Pallet, TrusteeSessionInfoLen, TrusteeSessionInfoOf, TrusteeSigRecord,
};
use frame_support::assert_ok;
use xp_assets_registrar::Chain;
use xp_protocol::X_BTC;

#[test]
fn test_do_trustee_election() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(TrusteeSessionInfoLen::<Test>::get(Chain::Bitcoin), 0);

        assert_eq!(Pallet::<Test>::do_trustee_election(), Ok(()));

        assert_eq!(TrusteeSessionInfoLen::<Test>::get(Chain::Bitcoin), 1);
    })
}

#[test]
fn test_claim_not_native_asset_reward() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(XGatewayCommon::do_trustee_election());

        TrusteeSigRecord::<Test>::mutate(bob(), |record| *record = 9);
        TrusteeSigRecord::<Test>::mutate(charlie(), |record| *record = 1);

        assert_eq!(XGatewayCommon::trustee_sig_record(bob()), 9);
        assert_eq!(XGatewayCommon::trustee_sig_record(charlie()), 1);
        assert_eq!(XGatewayCommon::trustee_sig_record(dave()), 0);

        let multi_address = XGatewayCommon::trustee_multisig_addr(Chain::Bitcoin).unwrap();

        assert_ok!(XGatewayRecords::deposit(&multi_address, X_BTC, 10));

        TrusteeSessionInfoOf::<Test>::mutate(Chain::Bitcoin, 1, |info| {
            if let Some(info) = info {
                info.0.trustee_list.iter_mut().for_each(|trustee| {
                    trustee.1 = XGatewayCommon::trustee_sig_record(&trustee.0);
                });
            }
        });

        assert_ok!(XGatewayCommon::apply_claim_trustee_reward(1));

        assert_eq!(Assets::balance(X_BTC, &bob()), 9);
        assert_eq!(Assets::balance(X_BTC, &charlie()), 1);
    });
}
