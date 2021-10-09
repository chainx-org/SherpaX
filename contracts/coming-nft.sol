// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "./precompile.sol";

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

    function mint(address to, uint256 cid) public _validCid(cid) {
        require(Coming.isOwnerOfCid(msg.sender, cid), "Mismatch cid onwer");

        _mint(to, cid);
    }

    function withdrawCid(address from, bytes32 substrate, uint256 cid) public {

        require(Coming.withdrawCid(from, substrate, cid), "Invalid WithdrawCid");

        _burn(cid);

        emit WithdrawCid(from, substrate, cid);
    }

    function withdrawBalance(address from, bytes32 substrate, uint256 value) public {
        require(Coming.withdrawBalance(from, substrate, value), "Invalid WithdrawBalance");

        emit WithdrawBalance(from, substrate, value);
    }

}
