// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

library Coming {
    address constant private precompile = address(0x401);
    // evm account transfer balance to substrate account
    // @from is the current owner of balance
    // @substrate is substrate account public key
    // @value is amount of balance
    function withdrawBalance(
        address from,
        bytes32 substrate,
        uint256 value
    ) public returns (bool) {
        (bool success, bytes memory returnData) = precompile.call(abi.encodePacked(from, substrate, value));

        // no data return
        delete returnData;

        return success;
    }

    // evm account transfer c-card to substrate account
    // @from is the current owner of c-card
    // @substrate is substrate account public key
    // @cid is the cid of c-card
    function withdrawCid(
        address from,
        bytes32 substrate,
        uint256 cid
    ) public returns (bool) {
        require(100_000 <= cid && cid < 1_000_000_000_000, "Require 100_000 <= cid < 1_000_000_000_000.");

        uint64 valid_cid = uint64(cid);

        (bool success, bytes memory returnData) = precompile.call(abi.encodePacked(from, substrate, valid_cid));

        // no data return
        delete returnData;

        return success;
    }

    // evm account transfer c-card to substrate account
    // @from is the current owner of c-card
    // @substrate is substrate account public key
    // @cid is the cid of c-card
    function isOwnerOfCid(
        address from,
        uint256 cid
    ) public view returns (bool) {
        require(100_000 <= cid && cid < 1_000_000_000_000, "Require 100_000 <= cid < 1_000_000_000_000.");

        uint64 valid_cid = uint64(cid);

        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(from, valid_cid));

        // no data return
        delete returnData;

        return success;
    }

}
