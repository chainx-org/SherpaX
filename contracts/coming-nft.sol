// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

// 2020.10.09
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/Strings.sol";

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

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));
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
        require(isOwnerOfCid(from, cid), "Mismatch Cid Owner");

        uint64 valid_cid = uint64(cid);

        (bool success, bytes memory returnData) = precompile.call(abi.encodePacked(from, substrate, valid_cid));

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));
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

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));
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

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));
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

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));
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
        require(isOperatorOfCid(operator, cid), "Mismatch Cid Operator");

        uint64 valid_cid = uint64(cid);

        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(operator, from, to, valid_cid));

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));

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
        require(isOwnerOfCid(owner, cid), "Mismatch Cid Owner");

        uint64 valid_cid = uint64(cid);

        (bool success, bytes memory returnData) = precompile.staticcall(abi.encodePacked(owner, to, valid_cid));

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));

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

        assembly {
            if eq(success, 0) {
                revert(add(returnData, 0x20), returndatasize())
            }
        }

        return abi.decode(returnData, (bool));
    }
}

contract ComingNFT is ERC721Enumerable {
    event WithdrawBalance(address indexed from, bytes32 indexed substrate, uint256 value);
    event WithdrawCid(address indexed from, bytes32 indexed substrate, uint256 cid);

    constructor() ERC721("ComingNFT", "c-card v1") {}

    modifier _validCid(uint256 cid) {
        require(
            100_000 <= cid && cid < 1_000_000_000_000,
            "Invalid cid."
        );
        _;
    }

    function _baseURI() internal pure override returns (string memory) {
        return "https://miniscan.coming.chat/NFTDetail?";
    }

    /**
     * @dev See {IERC721-approve}.
     */
    function approve(address to, uint256 cid) public override {
        address owner = ERC721.ownerOf(cid);
        require(to != owner, "ERC721: approval to current owner");

        require(
            _msgSender() == owner || isApprovedForAll(owner, _msgSender()),
            "ERC721: approve caller is not owner nor approved for all"
        );

        _approve(to, cid);

        require(Coming.approve(_msgSender(), to, cid), "Invalid Approve");
    }


    /**
     * @dev See {IERC721-setApprovalForAll}.
     */
    function setApprovalForAll(address operator, bool approved) public override {
        _setApprovalForAll(_msgSender(), operator, approved);

        require(Coming.setApprovalForAll(_msgSender(), operator, approved), "Invalid SetApprovalForAll");
    }


    /**
     * @dev See {IERC721-transferFrom}.
     */
    function transferFrom(
        address from,
        address to,
        uint256 cid
    ) public _validCid(cid) override {
        //solhint-disable-next-line max-line-length
        require(_isApprovedOrOwner(_msgSender(), cid), "ERC721: transfer caller is not owner nor approved");

        _transfer(from, to, cid);

        require(Coming.transferFrom(_msgSender(), from, to, cid), "Invalid TransferFrom");
    }

    /**
     * @dev See {IERC721-safeTransferFrom}.
     */
    function safeTransferFrom(
        address from,
        address to,
        uint256 cid,
        bytes memory _data
    ) public _validCid(cid) override {
        require(_isApprovedOrOwner(_msgSender(), cid), "ERC721: transfer caller is not owner nor approved");
        _safeTransfer(from, to, cid, _data);

        require(Coming.transferFrom(_msgSender(), from, to, cid), "Invalid SafeTransferFrom");
    }

    function isMatchApproval(
        uint256 cid
    ) public view returns (bool) {
        return Coming.isOperatorOfCid(getApproved(cid), cid);
    }

    function isMatchApprovalAll(
        address owner,
        address operator
    ) public view returns (bool) {
        bool inner = Coming.isApprovedForAll(owner, operator);

        return isApprovedForAll(owner, operator) == inner;
    }

    function mint(
        uint256 cid
    ) public _validCid(cid) {
        require(Coming.isOwnerOfCid(_msgSender(), cid), "Mismatch Cid Owner");

        _mint(_msgSender(), cid);
    }

    function withdrawCid(
        bytes32 substrate,
        uint256 cid
    ) public {

        require(Coming.withdrawCid(_msgSender(), substrate, cid), "Invalid WithdrawCid");

        _burn(cid);

        emit WithdrawCid(_msgSender(), substrate, cid);
    }

    function withdrawBalance(
        bytes32 substrate,
        uint256 value
    ) public {
        require(Coming.withdrawBalance(_msgSender(), substrate, value), "Invalid WithdrawBalance");

        emit WithdrawBalance(_msgSender(), substrate, value);
    }
}
