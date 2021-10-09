// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

// 2020.10.09
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/Strings.sol";
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

        //require(Coming.approve(_msgSender(), to, cid), "Invalid Approve");
    }


    /**
     * @dev See {IERC721-setApprovalForAll}.
     */
    function setApprovalForAll(address operator, bool approved) public override {
        _setApprovalForAll(_msgSender(), operator, approved);

        //require(Coming.setApprovalForAll(_msgSender(), operator, approved), "Invalid SetApprovalForAll");
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
        require(Coming.isOwnerOfCid(_msgSender(), cid), "Mismatch Cid owner");

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
