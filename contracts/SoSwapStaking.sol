// SPDX-License-Identifier: MIT
// OpenZeppelin Contracts v4.4.1 (token/ERC20/ERC20.sol)

/// (1) admin: deploy SoSwapToken with delay.
/// (2) admin: deploy SoSwapStaking with SoSwapToken contract address.
/// (3) admin: transfer ownership of the SoSwapToken to the SoSwapStaking in delay period.
/// (4) admin: approve LP for SoSwapStaking
/// (5) admin: add_pool_with_staking
/// (6) user : approve LP for SoSwapStaking
/// (7) user : stake LP
/// (8) user : unstake LP
/// (9) user : claim SO

pragma solidity ^0.8.0;

import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/IERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/utils/SafeERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/utils/math/SafeMath.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/access/Ownable.sol";

import "./SoSwapToken.sol";

contract SoSwapStaking is Ownable {
    using SafeMath for uint256;
    using SafeERC20 for IERC20;

    // Info of each user.
    struct UserInfo {
        // How many LP tokens the user has provided.
        uint256 amount;
        // Current acc_reward_per_share when stake/unstake/claim
        uint256 mark_reward_per_share;
        //
        // Basically, any point in time, the amount of SOs
        // entitled to a user but is pending to be distributed is:
        //
        //   pending reward = user.amount * (pool.acc_reward_per_share - user.mark_reward_per_share)
        //
        // Whenever a user stake or unstake LP tokens to a pool. Here's what happens:
        //   1. Mint the SOs to SoSwapStaking.
        //   2. The pool's `acc_reward_per_share` (and `last_reward_time`) gets updated.
        //   3. User receives the pending reward sent to his/her address.
        //   4. User's `amount` gets updated.
        //   5. User's `mark_reward_per_share` gets updated.
    }
    // Info of each pool.
    struct PoolInfo {
        // Address of LP token contract.
        IERC20 lp_token;
        // How many allocation points assigned to this pool.
        uint256 alloc_point;
        // Last block timestamp that SOs distribution occurs.
        uint256 last_reward_time;
        // Accumulated SOs per share, times 1e12. See below.
        uint256 acc_reward_per_share;
    }

    // The SOSWAP TOKEN!
    SoSwapToken public soswap;
    // All pools.
    PoolInfo[] public pool_info;
    // Info of each user that stakes LP tokens.
    mapping(uint256 => mapping(address => UserInfo)) public user_info;
    // Total allocation poitns. Must be the sum of all allocation points in all pools.
    uint256 public total_alloc_point = 0;

    // (account, pool_id, rewards, stake)
    event Stake(address indexed user, uint256 indexed pid, uint256 amount, uint256 stake);
    // (account, pool_id, rewards, unstake)
    event UnStake(address indexed user, uint256 indexed pid, uint256 amount, uint256 unstake);
    // (account, pool_id, rewards)
    event Claim(address indexed user, uint256 indexed pid, uint256 amount);
    // (account, pool_id, unstake)
    event EmergencyUnStake(address indexed user, uint256 indexed pid, uint256 unstake);

    constructor(SoSwapToken _soswap) {
        soswap = _soswap;
    }

    function _increment(uint256 i) internal pure returns (uint256) {
        unchecked { return i + 1; }
    }

    function pool_length() external view returns (uint256) {
        return pool_info.length;
    }

    function pool_exsist(IERC20 _lp_token) public view returns (bool) {
        for (uint256 i = 0; i < pool_info.length; i = _increment(i)) {
            if (pool_info[i].lp_token == _lp_token) {
                return true;
            }
        }

        return false;
    }

    // Add a new lp to the pool. Can only be called by the owner.
    function add_pool(
        IERC20 _lp_token,
        uint256 _alloc_point,
        bool _without_update
    ) public onlyOwner {
        require(!pool_exsist(_lp_token), "SoSwapStaking: pool is exsist");

        if (!_without_update) {
            // update all pools
            update_all_pools();
        }

        total_alloc_point = total_alloc_point.add(_alloc_point);

        pool_info.push(
            PoolInfo({
                lp_token: _lp_token,
                alloc_point: _alloc_point,
                last_reward_time: block.timestamp,
                acc_reward_per_share: 0
            })
        );
    }

    // Add a new lp to the pool with staking. Can only be called by the owner.
    function add_pool_with_staking(
        IERC20 _lp_token,
        uint256 _alloc_point,
        bool _without_update,
        uint256 _amount
    ) public onlyOwner {
        require(!pool_exsist(_lp_token), "SoSwapStaking: pool is exsist");

        if (!_without_update) {
            // update all pools
            update_all_pools();
        }

        total_alloc_point = total_alloc_point.add(_alloc_point);

        pool_info.push(
            PoolInfo({
                lp_token: _lp_token,
                alloc_point: _alloc_point,
                last_reward_time: block.timestamp,
                acc_reward_per_share: 0
            })
        );

        // pool_info.length >= 1
        uint256 _pid = pool_info.length - 1;

        _stake(msg.sender, _pid, _amount, false);
    }

    // Update the given pool's SO allocation point. Can only be called by the owner.
    function set_pool(
        uint256 _pid,
        uint256 _alloc_point,
        bool _without_update
    ) public onlyOwner {
        require(_pid < pool_info.length, "SoSwapStaking: pool is not exsist");

        if (!_without_update) {
            // update all pools
            update_all_pools();
        }

        total_alloc_point = total_alloc_point
            .sub(pool_info[_pid].alloc_point)
            .add(_alloc_point);

        pool_info[_pid].alloc_point = _alloc_point;
    }

    // Update reward vairables for all pools. Be careful of gas spending!
    function update_all_pools() public {
        for (uint256 pid = 0; pid < pool_info.length; pid = _increment(pid)) {
            update_pool(pid);
        }
    }

    // Update reward variables of the given pool to be up-to-date.
    function update_pool(uint256 _pid) public {
        PoolInfo storage pool = pool_info[_pid];
        uint256 lp_supply = pool.lp_token.balanceOf(address(this));

        if (block.timestamp <= pool.last_reward_time || total_alloc_point == 0 || lp_supply == 0) {
            pool.last_reward_time = block.timestamp;
            return;
        }

        uint256 so_rewards = soswap
            .calculate_rewards(pool.last_reward_time, block.timestamp)
            .mul(pool.alloc_point)
            .div(total_alloc_point);

        if (soswap.can_mint() && soswap.ready_mint()) {
            soswap.mint();
        }

        pool.acc_reward_per_share = pool.acc_reward_per_share
            .add(so_rewards.mul(1e12)
            .div(lp_supply));

        pool.last_reward_time = block.timestamp;
    }

    // Stake LP tokens to SoSwapStaking for SoSwapToken allocation.
    function stake(uint256 _pid, uint256 _amount) public {
        _stake(msg.sender, _pid, _amount, true);
    }

    function _stake(address _account, uint256 _pid, uint256 _amount, bool update) internal {
        require(_pid < pool_info.length, "SoSwapStaking: pool is not exsist");
        require(_amount > 0, "SoSwapStaking: amount is 0");

        PoolInfo storage pool = pool_info[_pid];
        UserInfo storage user = user_info[_pid][_account];
        uint256 pending = 0;

        if (update) {
            update_pool(_pid);
        }

        if (user.amount > 0 && pool.acc_reward_per_share > user.mark_reward_per_share) {
            pending = user.amount
                .mul(pool.acc_reward_per_share.sub(user.mark_reward_per_share))
                .div(1e12);

            safe_so_transfer(_account, pending);
        }

        pool.lp_token.safeTransferFrom(
            address(_account),
            address(this),
            _amount
        );

        user.amount = user.amount.add(_amount);
        user.mark_reward_per_share = pool.acc_reward_per_share;

        emit Stake(_account, _pid, pending, _amount);
    }

    // UnStake LP tokens from SoSwapStaking.
    function unstake(uint256 _pid, uint256 _amount) public {
        require(_pid < pool_info.length, "SoSwapStaking: pool is not exsist");
        require(_amount > 0, "SoSwapStaking: amount is 0");

        PoolInfo storage pool = pool_info[_pid];
        UserInfo storage user = user_info[_pid][msg.sender];
        uint256 pending = 0;

        require(user.amount >= _amount, "SoSwapStaking: more than can unstake");

        update_pool(_pid);

        if (user.amount > 0 && pool.acc_reward_per_share > user.mark_reward_per_share) {
            pending = user.amount
                .mul(pool.acc_reward_per_share.sub(user.mark_reward_per_share))
                .div(1e12);

            safe_so_transfer(msg.sender, pending);
        }


        pool.lp_token.safeTransfer(address(msg.sender), _amount);

        user.amount = user.amount.sub(_amount);
        user.mark_reward_per_share = pool.acc_reward_per_share;

        emit UnStake(msg.sender, _pid, pending, _amount);
    }

    // Claim SoSwapToken rewards from SoSwapStaking.
    function claim(uint256 _pid) public {
        require(_pid < pool_info.length, "SoSwapStaking: pool is not exsist");

        PoolInfo storage pool = pool_info[_pid];
        UserInfo storage user = user_info[_pid][msg.sender];

        update_pool(_pid);

        uint256 pending = user.amount
            .mul(pool.acc_reward_per_share.sub(user.mark_reward_per_share))
            .div(1e12);

        safe_so_transfer(msg.sender, pending);

        user.mark_reward_per_share = pool.acc_reward_per_share;

        emit Claim(msg.sender, _pid, pending);
    }

    // View function to see pending SoSwapTokens on frontend.
    function pendingSoSwap(uint256 _pid, address _user) public view returns (uint256) {
        require(_pid < pool_info.length, "SoSwapStaking: pool is not exsist");

        PoolInfo storage pool = pool_info[_pid];
        UserInfo storage user = user_info[_pid][_user];

        uint256 acc_reward_per_share = pool.acc_reward_per_share;
        uint256 lp_supply = pool.lp_token.balanceOf(address(this));

        if (total_alloc_point == 0 || lp_supply == 0) {
            return 0;
        }

        if (block.timestamp > pool.last_reward_time) {
            uint256 so_rewards = soswap
                .calculate_rewards(pool.last_reward_time, block.timestamp)
                .mul(pool.alloc_point)
                .div(total_alloc_point);

            acc_reward_per_share = acc_reward_per_share.add(
                so_rewards.mul(1e12).div(lp_supply)
            );
        }

        return user.amount.mul(acc_reward_per_share.sub(user.mark_reward_per_share)).div(1e12);
    }

    // EMERGENCY ONLY.
    // UnStake without caring about rewards which will be locked in SoSwapStaking.
    function emergency_unstake(uint256 _pid) public {
        require(_pid < pool_info.length, "SoSwapStaking: pool is not exsist");

        PoolInfo storage pool = pool_info[_pid];
        UserInfo storage user = user_info[_pid][msg.sender];

        pool.lp_token.safeTransfer(address(msg.sender), user.amount);

        user.amount = 0;
        user.mark_reward_per_share = 0;

        emit EmergencyUnStake(msg.sender, _pid, user.amount);
    }

    // Safe SoSwapToken transfer function.
    // Just in case if rounding error causes pool to not have enough SoSwapToken.
    function safe_so_transfer(address _to, uint256 _amount) internal {
        uint256 soswap_balance = soswap.balanceOf(address(this));
        if (_amount > soswap_balance) {
            soswap.transfer(_to, soswap_balance);
        } else {
            soswap.transfer(_to, _amount);
        }
    }
}
