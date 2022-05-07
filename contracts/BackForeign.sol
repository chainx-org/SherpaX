// SPDX-License-Identifier: MIT
// OpenZeppelin Contracts v4.4.1 (token/ERC20/ERC20.sol)

pragma solidity ^0.8.0;

import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/IERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/utils/SafeERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/access/Ownable.sol";

/// workflow:
/// (1) admin(onchain): set_foreign
/// (2) user(onchain): approve BackForeign contract
/// (3) user(onchain): back_foreign
/// (4) admin(offchain): validate the request
/// (5) admin(onchain): burn the tmp_burn's token
/// (6) admin(foreign chain): transfer the token to destination on the foreign chain

contract BackForeign is Context, Ownable {
    using SafeERC20 for IERC20;

    // First: account transfer the foreign tokens to `tmp_burn`,
    // Then: the admin of these tokens will actually burn the tokens of `tmp_burn`.
    address public constant tmp_burn = 0x2222222222222222222222222222222222222222;

    uint256 public index = 0;

    mapping(IERC20 => bool) public foreign_map;

    // (index, erc20, amount, destination)
    event Back(uint256 indexed index, IERC20 indexed erc20, uint256 amount, bytes destination);

    // Admin: set allow back foregin tokens
    function set_foreign(IERC20 _erc20, bool _is_active) public onlyOwner {
        foreign_map[_erc20] = _is_active;
    }

    // User: transfer back the tokens to the foreign chain
    // @dev: validate the `_destination` offchain.
    function back_foreign(IERC20 _erc20, uint256 _amount, string memory _destination) public {
        require(foreign_map[_erc20], "BackForeign: inactive erc20");
        require(bytes(_destination).length > 0 && bytes(_destination).length < 256, "BackForeign: invalid destination string");

        // 1. transfer token to tmp_burn
        _erc20.safeTransferFrom(address(msg.sender), address(tmp_burn), _amount);

        // 2. update index
        index = index + 1;

        // 3. event
        emit Back(index, _erc20, _amount, bytes(_destination));
    }
}
