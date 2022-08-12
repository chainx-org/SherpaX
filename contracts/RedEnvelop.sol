// SPDX-License-Identifier: MIT

import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/IERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/utils/SafeERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/access/Ownable.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/utils/Context.sol";

pragma solidity ^0.8.0;

contract RedEnvelop is Ownable {
    using SafeERC20 for IERC20;

    struct RedEnvelopInfo {
        IERC20  token;
        uint256 remain_count;
        uint256 remain_balance;
    }

    uint256 public constant max_count = 1000;
    uint256 public base_prepaid_fee;
    address public admin;
    address public beneficiary;
    uint256 public next_id;

    mapping (uint256 => RedEnvelopInfo) public red_envelop_infos;

    event AdminChanged(address _old, address _new);
    event BeneficiaryChanged(address _old, address _new);
    event NewBasePrepaidFee(uint256 _fee);
    event NewRedEnvelop(uint256 _id, IERC20 _token, uint256 _count, uint256 _balance);
    event UpdateRedEnvelop(uint256 _id, uint256 _remain_count, uint256 _remain_balance);

    modifier onlyAdmin() {
        require(_msgSender() == admin, "RedEnvelop: require admin");
        _;
    }

    constructor(address _admin, address _beneficiary, uint256 _base_fee) {
        admin = _admin;
        beneficiary = _beneficiary;
        base_prepaid_fee = _base_fee;
        next_id = 0;
    }

    // call by owner
    function set_admin(address new_admin) public onlyOwner {
        address old_admin = admin;
        admin = new_admin;

        emit AdminChanged(old_admin, new_admin);
    }

    // call by owner
    function set_beneficiary(address new_beneficiary) public onlyOwner {
        address old_beneficiary = new_beneficiary;
        beneficiary = new_beneficiary;

        emit BeneficiaryChanged(old_beneficiary, new_beneficiary);
    }

    // call by admin
    function set_prepaid_fee(uint256 new_fee) public onlyAdmin {
        base_prepaid_fee = new_fee;

        emit NewBasePrepaidFee(new_fee);
    }

    // call by anyone
    function create(
        IERC20  token,
        uint256 count,
        uint256 total_balance
    ) public payable {
        require(count > 0 && count <= max_count, "RedEnvelop: invalid count");
        uint256 prepaid_fees = calc_prepaid_fee(count);
        require(msg.value >= prepaid_fees, "RedEnvelop: invalid payable value");

        RedEnvelopInfo storage info = red_envelop_infos[next_id];

        // Prepaid gas_fee to admin for opening red envelops.
        payable(admin).transfer(prepaid_fees);

        // Refund any extra payment.
        if (msg.value > prepaid_fees) {
            payable(_msgSender()).transfer(msg.value - prepaid_fees);
        }

        token.safeTransferFrom(
            address(_msgSender()),
            address(this),
            total_balance
        );

        info.token = token;
        info.remain_count = count;
        info.remain_balance = total_balance;

        emit NewRedEnvelop(next_id, token, count, total_balance);

        next_id = next_id + 1;
    }


    // call by admin
    // note: deduplicate and validate account offchain
    function open(
        uint256 id,
        address[] memory luck_accounts,
        uint256[] memory balances
    ) public onlyAdmin {
        require(is_valid(id), "RedEnvelop: invalid id");
        require(luck_accounts.length == balances.length, "RedEnvelop: mismatch accounts and balances");
        require(red_envelop_infos[id].remain_count >= luck_accounts.length, "RedEnvelop: too many accounts");

        uint256 sum = 0;
        for (uint i = 0; i < balances.length; i++) {
            sum += balances[i];
        }
        require(red_envelop_infos[id].remain_balance >= sum, "RedEnvelop: more than remain balance");

        uint256 contract_balance = red_envelop_infos[id].token.balanceOf(address(this));
        require(red_envelop_infos[id].remain_balance <= contract_balance, "RedEnvelop: insufficient token balance");

        red_envelop_infos[id].remain_count -= balances.length;
        red_envelop_infos[id].remain_balance -= sum;

        batch_token_transfer(red_envelop_infos[id].token, luck_accounts, balances);

        emit UpdateRedEnvelop(id, red_envelop_infos[id].remain_count, red_envelop_infos[id].remain_balance);
    }

    // call by admin
    function close(uint256 id, address maybe_creator) public onlyAdmin {
        require(is_valid(id), "RedEnvelop: invalid id");

        uint256 remain_balance = red_envelop_infos[id].remain_balance;
        if (remain_balance > 0) {

            if (maybe_creator != address(0)) {
                // transfer to creator
                red_envelop_infos[id].token.transfer(maybe_creator, remain_balance);
            } else {
                // transfer to owner
                red_envelop_infos[id].token.transfer(beneficiary, remain_balance);
            }

            emit UpdateRedEnvelop(id, 0, 0);
        }

        delete red_envelop_infos[id];
    }

    // call by anyone
    function is_valid(uint256 id) public view returns (bool) {
        return (address(red_envelop_infos[id].token) != address(0));
    }

    // call by anyone
    function calc_prepaid_fee(uint256 count) public view returns (uint256) {
        if (count <= 10) { // [0, 10]
            return 4 * base_prepaid_fee;
        } else if (count > 10 && count <= 100) { // (10, 100]
            return 16 * base_prepaid_fee;
        } else { // (100, 1000]
            return 200 * base_prepaid_fee;
        }
    }

    function batch_token_transfer(
        IERC20 token,
        address[] memory accounts,
        uint256[] memory balances
    ) internal
    {
        for (uint i = 0; i < accounts.length; i++) {
            if (accounts[i] != address(0)) {
                token.safeTransfer(accounts[i], balances[i]);
            }
        }
    }
}
