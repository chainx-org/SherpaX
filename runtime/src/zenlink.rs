// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use zenlink_protocol::{
    make_x2_location, NativeCurrencyAdaptor
};

use super::{
    parameter_types, vec, AccountId, Balances, Convert,
    Event, LocationConverter, ModuleId, MultiLocation, ParachainInfo,
    Runtime, Vec, XcmConfig, XcmExecutor,
};

parameter_types! {
    pub const ZenlinkModuleId: ModuleId = ModuleId(*b"zenlink1");
    pub ZenlinkRegistedParaChains: Vec<(MultiLocation, u128)> = vec![
        // Phala local and live, 1 PHA
        (make_x2_location(30),    1_000_000_000_000),
        // Sherpax live
        (make_x2_location(59),  500),
        // Bifrost local and live, 0.01 BNC
        (make_x2_location(107),   10_000_000_000),
        // Zenlink live
        (make_x2_location(188), 500),
        // Zenlink local
        (make_x2_location(200), 500),
        // Sherpax local
        (make_x2_location(300), 500),
        // Plasm local and live, 0.001 PLM
        (make_x2_location(5000), 1_000_000_000_000)
    ];
}

pub struct AccountId32Converter;

impl Convert<AccountId, [u8; 32]> for AccountId32Converter {
    fn convert(account_id: AccountId) -> [u8; 32] {
        account_id.into()
    }
}


impl zenlink_protocol::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type AccountIdConverter = LocationConverter;
    type AccountId32Converter = AccountId32Converter;
    type ParaId = ParachainInfo;
    type ModuleId = ZenlinkModuleId;
    type TargetChains = ZenlinkRegistedParaChains;
    type NativeCurrency = NativeCurrencyAdaptor<Runtime, Balances>;
    type OtherAssets = ();
}
