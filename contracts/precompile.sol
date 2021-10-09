// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

library Coming {
    address constant private precompile = address(0x401);
    // input = 20 + 32 + 32 = 84 bytes
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

    // input = 20 + 32 + 8 = 60 bytes
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

    // input = 20 + 8 = 28 bytes
    // check if mapping account of the specified address and the owner of c-card match
    // @from is the specified address
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

    // input = 20 + 8 + 1 padding = 29 bytes
    // check if mapping account of the specified address and the operator of c-card match
    // @from is the specified address
    // @cid is the cid of c-card
    function isOperatorOfCid(
        address from,
        uint256 cid
    ) public view returns (bool) {
        require(100_000 <= cid && cid < 1_000_000_000_000, "Require 100_000 <= cid < 1_000_000_000_000.");
        uint64 valid_cid = uint64(cid);
        bool padding = true;

        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(from, valid_cid, padding));

        // no data return
        delete returnData;

        return success;
    }

    // input = 20 + 20 = 40 bytes
    // check if inner owner and operator match
    // @owner is the evm address
    // @operator is the evm address
    function isApprovedForAll(
        address owner,
        address operator
    ) public view returns (bool) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(owner, operator));

        // no data return
        delete returnData;

        return success;
    }

    // input = 20 + 20 + 20 + 8= 68 bytes
    // transferFrom c-card between evm account
    // @operator is spender of c-card
    // @from is the owner of c-card
    // @to is the receiver of c-card
    // @cid is the cid of c-card
    function transferFrom(
        address operator,
        address from,
        address to,
        uint256 cid
    ) public view returns (bool) {
        require(100_000 <= cid && cid < 1_000_000_000_000, "Require 100_000 <= cid < 1_000_000_000_000.");
        uint64 valid_cid = uint64(cid);

        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(operator, from, to, valid_cid));

        // no data return
        delete returnData;

        return success;

    }

    // input = 20 + 20 + 8= 48 bytes
    // gives permission to `to` to transfer `cid` c-card to another account
    // @owner is the owner of c-card
    // @to is the spender of c-card
    // @cid is the cid of c-card
    function approve (
        address owner,
        address to,
        uint256 cid
    ) public view returns (bool) {
        require(100_000 <= cid && cid < 1_000_000_000_000, "Require 100_000 <= cid < 1_000_000_000_000.");
        uint64 valid_cid = uint64(cid);

        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(owner, to, valid_cid));

        // no data return
        delete returnData;

        return success;

    }

    // input = 20 + 20 + 1= 41 bytes
    // approve or remove `operator` as an operator for the caller
    // @owner is the owner of c-card
    // @operator is the spender of c-card
    // @approved is true or false
    function setApprovalForAll (
        address owner,
        address operator,
        bool approved
    ) public view returns (bool) {
        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(owner, operator, approved));

        // no data return
        delete returnData;

        return success;
    }
}
