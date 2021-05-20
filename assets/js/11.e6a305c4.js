(window.webpackJsonp=window.webpackJsonp||[]).push([[11],{366:function(e,t,s){"use strict";s.r(t);var r=s(44),o=Object(r.a)({},(function(){var e=this,t=e.$createElement,s=e._self._c||t;return s("ContentSlotsDistributor",{attrs:{"slot-key":e.$parent.slotKey}},[s("h1",{attrs:{id:"xgatewaybitocinbridge"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#xgatewaybitocinbridge"}},[e._v("#")]),e._v(" XGatewayBitocinBridge")]),e._v(" "),s("h2",{attrs:{id:"storage"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#storage"}},[e._v("#")]),e._v(" Storage")]),e._v(" "),s("div",{staticClass:"custom-block tip"},[s("p",{staticClass:"custom-block-title"},[e._v("TIP")]),e._v(" "),s("p",[s("code",[e._v("module")]),e._v(" is "),s("code",[e._v("xGatewayBitcoinBridge")]),e._v(" or "),s("code",[e._v("xGatewayDogecoinBridge")]),e._v(" depends on which network used.")])]),e._v(" "),s("div",{staticClass:"custom-block tip"},[s("p",{staticClass:"custom-block-title"},[e._v("TIP")]),e._v(" "),s("p",[e._v("Please use polkadotjs "),s("code",[e._v("api.query")]),e._v(", no extra rpc provided. Refer to "),s("a",{attrs:{href:"https://polkadot.js.org",target:"_blank",rel:"noopener noreferrer"}},[e._v("https://polkadot.js.org"),s("OutboundLink")],1),e._v(" for help.")])]),e._v(" "),s("div",{staticClass:"custom-block tip"},[s("p",{staticClass:"custom-block-title"},[e._v("TIP")]),e._v(" "),s("p",[e._v("All types below are provided within "),s("a",{attrs:{href:"https://github.com/chainx-org/Chainx-Bridge-Front/tree/main/src/interfaces",target:"_blank",rel:"noopener noreferrer"}},[s("code",[e._v("interfaces")]),s("OutboundLink")],1),e._v(" module.")])]),e._v(" "),s("p",[s("strong",[e._v("Collaterals")]),e._v("\nCollaterals for "),s("code",[e._v("account")])]),e._v(" "),s("ul",[s("li",[s("code",[e._v("account: AccountId")])]),e._v(" "),s("li",[e._v("return: "),s("code",[e._v("Balace")])])]),e._v(" "),s("p",[s("strong",[e._v("ExchangeRate")]),e._v("\nExchange rate from pcx to btc")]),e._v(" "),s("p",[s("strong",[e._v("BridgeStatus")]),e._v("\nGet bridge status")]),e._v(" "),s("p",[s("strong",[e._v("Vaults")]),e._v("\nQuery vault")]),e._v(" "),s("ul",[s("li",[s("code",[e._v("account: AccountId")])]),e._v(" "),s("li",[e._v("return "),s("code",[e._v("Option<Vault>")])])]),e._v(" "),s("p",[s("strong",[e._v("OuterAddresses")]),e._v("\nStorage for outer chain's addresses.")]),e._v(" "),s("ul",[s("li",[s("code",[e._v("addr_str: Text")])]),e._v(" "),s("li",[e._v("return "),s("code",[e._v("Option<AccountId>")])])]),e._v(" "),s("p",[s("strong",[e._v("IssueGriefingFee")]),e._v("\nCollateral that requester should be reserved when issuing.")]),e._v(" "),s("ul",[s("li",[e._v("return "),s("code",[e._v("Percent")])])]),e._v(" "),s("p",[s("strong",[e._v("IssueRequests")]),e._v("\nIssue request collections")]),e._v(" "),s("ul",[s("li",[s("code",[e._v("id: RequestId")])]),e._v(" "),s("li",[e._v("return "),s("code",[e._v("IssueRequest")])])]),e._v(" "),s("p",[s("strong",[e._v("RedeemRequests")]),e._v("\nRedeem request collections")]),e._v(" "),s("ul",[s("li",[s("code",[e._v("id: RequestId")])]),e._v(" "),s("li",[e._v("return "),s("code",[e._v("RedeemRequest")])])]),e._v(" "),s("h2",{attrs:{id:"extrinsics"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#extrinsics"}},[e._v("#")]),e._v(" Extrinsics")]),e._v(" "),s("p",[s("strong",[e._v("registerVault")])]),e._v(" "),s("ul",[s("li",[s("code",[e._v("collateral: Balance")])]),e._v(" "),s("li",[s("code",[e._v("addrStr: Text")])])]),e._v(" "),s("p",[s("strong",[e._v("addExtraCollateral")])]),e._v(" "),s("p",[e._v("Add collateral for requester if requester is vault.")]),e._v(" "),s("ul",[s("li",[s("code",[e._v("collateral: Balance")])])]),e._v(" "),s("p",[s("strong",[e._v("requestIssue")])]),e._v(" "),s("p",[e._v("Request issuing "),s("code",[e._v("amount")]),e._v(" bridge target asset. -"),s("code",[e._v("vaultId: AccountId")]),e._v(" -"),s("code",[e._v("amount: Balance")])]),e._v(" "),s("p",[s("strong",[e._v("cancelRequest")])]),e._v(" "),s("p",[e._v("Cancel an issue request if it's expired. -"),s("code",[e._v("requestId: RequestId")])]),e._v(" "),s("p",[s("strong",[e._v("requestRedeem")])]),e._v(" "),s("ul",[s("li",[s("code",[e._v("vaultId: AccountId")])]),e._v(" "),s("li",[s("code",[e._v("amount: Balance")])]),e._v(" "),s("li",[s("code",[e._v("outerAddress: Text")]),e._v(": address that vault should transfer to.")])]),e._v(" "),s("p",[s("strong",[e._v("cancelRedeem")])]),e._v(" "),s("ul",[s("li",[s("code",[e._v("requestId: RequestId")])]),e._v(" "),s("li",[s("code",[e._v("reimburse: bool")]),e._v(": "),s("code",[e._v("true")]),e._v(" for force redeem, "),s("code",[e._v("false")]),e._v(" otherwise.")])])])}),[],!1,null,null,null);t.default=o.exports}}]);