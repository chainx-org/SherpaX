// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use frame_support::traits::{
    Box, Currency, ExistenceRequirement, ExistenceRequirement::KeepAlive, WithdrawReasons,
};
use sp_runtime::{DispatchResult, SaturatedConversion};

use zenlink_protocol::{
    AccountId32Aliases, Junction, LocationInverter, MultiLocation, NetworkId, OperationalAsset,
    Origin as ZenlinkOrigin, ParaChainWhiteList, ParentIsDefault, RelayChainAsNative, Sibling,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SovereignSignedViaLocation, TokenBalance, Transactor, XcmCfg, XcmExecutor,
};

use super::{
    parameter_types, vec, AccountId, Balances, Call, Convert, Event, ModuleId, Origin,
    ParachainInfo, ParachainSystem, Runtime, Vec, ZenlinkProtocol,
};

parameter_types! {
    pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
    pub const DEXModuleId: ModuleId = ModuleId(*b"zenlink1");
    pub RelayChainOrigin: Origin = ZenlinkOrigin::Relay.into();
    pub Ancestry: MultiLocation = Junction::Parachain {
        id: ParachainInfo::parachain_id().into()
    }.into();

    pub SiblingParachains: Vec<MultiLocation> = vec![
        // Phala local and live
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 30 }),
        // Sherpax live
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 59 }),
        // Bifrost local and live
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 107 }),
        // Zenlink live
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 188 }),
        // Zenlink local
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 200 }),
        // Sherpax local
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 300 })
    ];
}

pub struct AccountId32Converter;

impl Convert<AccountId, [u8; 32]> for AccountId32Converter {
    fn convert(account_id: AccountId) -> [u8; 32] {
        account_id.into()
    }
}

type LocationConverter = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RococoNetwork, AccountId>,
);

pub type LocalAssetTransactor =
    Transactor<ZenlinkProtocol, LocationConverter, AccountId, ParachainInfo>;

type LocalOriginConverter = (
    SovereignSignedViaLocation<LocationConverter, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<ZenlinkOrigin, Origin>,
    SignedAccountId32AsNative<RococoNetwork, Origin>,
);

pub struct XcmConfig;

impl XcmCfg for XcmConfig {
    type Call = Call;
    type XcmSender = ZenlinkProtocol;
    // How to withdraw and deposit an asset.
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = LocalOriginConverter;
    type IsReserve = ParaChainWhiteList<SiblingParachains>;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
}

impl zenlink_protocol::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type UpwardMessageSender = ParachainSystem;
    type XcmpMessageSender = ParachainSystem;
    type AccountIdConverter = LocationConverter;
    type AccountId32Converter = AccountId32Converter;
    type ParaId = ParachainInfo;
    type ModuleId = DEXModuleId;
    type TargetChains = SiblingParachains;
    type AssetModuleRegistry = AssetModuleRegistry;
}

/// A proxy struct implement `OperationalAsset`. It control `Balance` module
struct BalancesProxy;

impl OperationalAsset<u32, AccountId, TokenBalance> for BalancesProxy {
    fn balance(&self, _id: u32, who: AccountId) -> u128 {
        Balances::free_balance(&who).saturated_into::<TokenBalance>()
    }

    fn total_supply(&self, _id: u32) -> u128 {
        Balances::total_issuance().saturated_into::<TokenBalance>()
    }

    fn inner_transfer(
        &self,
        _id: u32,
        origin: AccountId,
        target: AccountId,
        amount: u128,
    ) -> DispatchResult {
        <Balances as Currency<AccountId>>::transfer(&origin, &target, amount, KeepAlive)
    }

    #[allow(unused_must_use)]
    fn inner_deposit(&self, _id: u32, origin: AccountId, amount: u128) -> DispatchResult {
        <Balances as Currency<AccountId>>::deposit_creating(&origin, amount);

        Ok(())
    }

    #[allow(unused_must_use)]
    fn inner_withdraw(&self, _id: u32, origin: AccountId, amount: u128) -> DispatchResult {
        <Balances as Currency<AccountId>>::withdraw(
            &origin,
            amount,
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        )?;

        Ok(())
    }
}

parameter_types! {
    /// Zenlink protocol use the proxy in the registry to control assets module.
    /// The first in the tuple represent the module index.
    pub AssetModuleRegistry : Vec<(u8, Box<dyn OperationalAsset<u32, AccountId, TokenBalance>>)> = vec![
        (2u8, Box::new(BalancesProxy))
    ];
}
