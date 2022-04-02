// SPDX-License-Identifier: MIT
// OpenZeppelin Contracts v4.4.1 (token/ERC20/ERC20.sol)

pragma solidity ^0.8.0;

import "<https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/ERC20.sol>";
import "<https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/access/Ownable.sol>";

contract SoSwapToken is ERC20("SoSwapToken", "SO"), Ownable {
    /// 0.05 SO per minutes
    uint256 constant initial_reward = 50000000000000000;
    /// 60s = 1 minutes
    uint256 constant minium_interval = 60;
    /// ~4 years
    uint256 constant halving_period = 2100000 * minium_interval;

    uint256 private _genesis_timestamp;
    uint256 private _last_mint;

    modifier ReadyMint() {
        require(block.timestamp > _genesis_timestamp, "SoSwapToken: not ready yet");

        _;
    }

    constructor(uint256 delay) {
        _genesis_timestamp = block.timestamp + delay;
    }

    // Used for SoSwapStaking check
    function can_mint() public view returns(bool) {
        return _msgSender() == owner();
    }

    // 50000000000000000 >> 56 == 0
    function halving_reward(uint256 _times) public pure returns(uint256) {
        return initial_reward >> _times;
    }

    // Calculate SoSwapToken rewards for a given time period
    // (start_time, end_time]
    function calculate_rewards(uint256 _start, uint256 _end) public view returns(uint256) {
        uint256 start = _start < _genesis_timestamp ? _genesis_timestamp : _start;
        if (_end <= start) {
            return 0;
        }

        uint256 start_offset = start - _genesis_timestamp;
        uint256 end_offset = _end - _genesis_timestamp;
        uint256 n1 = start_offset / halving_period;
        uint256 n2 = end_offset / halving_period;

        /// because _end > start, so n2 >= n1

        // 1. n2 == n1
        if (n2 == n1) {
            return (end_offset / minium_interval - start_offset / minium_interval) * halving_reward(n1);
        }

        // 2. n2 > n1
        // 2.1 first section
        uint256 part = (halving_period / minium_interval - start_offset % halving_period / minium_interval);
        uint256 first_section = part * halving_reward(n1);

        // 2.2 second section
        uint256 second_section = halving_period / minium_interval  * (halving_reward(n1) - halving_reward(n2 - 1));

        // 2.3 third section
        uint256 other_part = end_offset % halving_period / minium_interval;
        uint256 third_section =  other_part * halving_reward(n2);

        return first_section + second_section + third_section;
    }

    function genesis_timestamp() public view returns(uint256) {
        return _genesis_timestamp;
    }

    function last_mint() public view returns(uint256) {
        return _last_mint;
    }

    // Called by the owner(SoSwapStaking).
    function mint() public onlyOwner ReadyMint returns(uint256) {
        uint256 amount = calculate_rewards(_last_mint, block.timestamp);

        // 1 SO = 1000000000000000000.
        // Hard Cap is 210000 SO.
        if (amount != 0 && totalSupply() + amount <= 210000000000000000000000) {
            _mint(owner(), amount);
        }

        _last_mint = block.timestamp;

        return amount;
    }
}

