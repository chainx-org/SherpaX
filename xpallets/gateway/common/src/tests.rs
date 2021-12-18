// Copyright 2019-2020 ChainX Project Authors. Licensed under GPL-3.0.

use frame_system::RawOrigin;
use xp_assets_registrar::Chain;
use crate::{LittleBlackHouse, mock::{ExtBuilder, Test}, Pallet, TrusteeAdmin, TrusteeSessionInfoLen, TrusteeSessionInfoOf};
use crate::mock::{alice, bob, charlie, dave};

#[test]
fn test_do_trustee_election() {
    ExtBuilder::default().build().execute_with(|| {
        let dave = (
            dave(),
            b"".to_vec(),
            hex::decode("029f9830fe29e28064ee2ee57423f000146b75f7f92131d9089e5b395f6e51daf7")
                .expect("hex decode failed"),
            hex::decode("033ad05ed2677f49c9591a7c273b5d13afb26c2e964deee403178c053e2149a1fd")
                .expect("hex decode failed"),
        );
        Pallet::<Test>::setup_trustee_impl(
                            dave.0.clone(),
                            None,
                            Chain::Bitcoin,
                            dave.1,
                            dave.2,
                            dave.3,
                        );
        TrusteeAdmin::<Test>::put(alice());
        assert_eq!(TrusteeSessionInfoLen::<Test>::get(Chain::Bitcoin), 0);

        assert_eq!(Pallet::<Test>::move_trust_to_black_room(RawOrigin::Signed(alice()).into(), Some(vec![charlie()])), Ok(()));
        assert_eq!(LittleBlackHouse::<Test>::get(), vec![charlie()]);
        assert_eq!(TrusteeSessionInfoLen::<Test>::get(Chain::Bitcoin), 1);
        let info = TrusteeSessionInfoOf::<Test>::get(Chain::Bitcoin, 1).unwrap();
        let trustee = info.0.trustee_list;
        assert_eq!(trustee, vec![(dave.0, 0),  (bob(), 0), (alice(), 0)]);
    })
}
