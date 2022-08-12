// SPDX-License-Identifier: MIT

import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/IERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/token/ERC20/utils/SafeERC20.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/access/Ownable.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.1/contracts/utils/Context.sol";

pragma solidity ^0.8.0;

contract LuckyTreasure is Ownable {
    using SafeERC20 for IERC20;

    struct TreasureInfo {
        IERC20  treasure_token;
        IERC20  ticket_token;
        uint256 treasure_balance;
        uint256 ticket_balance;
        uint256 ticket_amount;
        uint256 expired_time;
        address creator;
        address winner;
        bool    refunded_all;
    }

    struct ParticipantInfo {
        bytes32 secret_hash;
        uint256 remain_tickets;
        // every time participate is called, the last_hash is calculated
        bytes32 last_hash;
        address[] accounts;
    }

    struct RefundInfo {
        uint256 refund_time;
        uint256 remain_accounts;
        mapping (address => uint256) balances;
    }

    uint256 public constant max_tickets = 1000;

    uint256 public max_duration;
    uint256 public fee_point;
    uint256 public slash_point;
    address public admin;
    uint256 public next_id;

    mapping (uint256 => TreasureInfo) public treasure_infos;
    mapping (uint256 => ParticipantInfo) public participant_infos;
    mapping (uint256 => RefundInfo) public refund_infos;

    event AdminChanged(address oldAdmin, address newAdmin);
    event NewPoint(uint256 fee_point, uint256 slash_point);
    event NewTreasure(uint256 id);
    event NewDuration(uint256 duration);
    event Participate(uint256 id, address participant, uint256 tickets);
    event Winner(uint256 id, address winner);
    event Slash(uint256 id, uint256 treasure_balance);
    event RefundAll(uint256 id);
    event Refund(uint256 id, address participant, uint256 treasure_balance);

    modifier onlyAdmin() {
        require(_msgSender() == admin, "LuckyTreasure: require admin");
        _;
    }

    constructor(address _admin) {
        admin = _admin;
        next_id = 0;

        // one day
        max_duration = 1 days;
        // 2.5 %
        fee_point = 250;
        // 10 %
        slash_point = 1000;
    }

    // call by owner
    function set_admin(address new_admin) public onlyOwner {
        address old_admin = admin;
        admin = new_admin;

        emit AdminChanged(old_admin, new_admin);
    }

    // call by admin
    function set_point(uint256 _fee_point, uint256 _slash_point) public onlyAdmin {
        require(0 < _fee_point && _fee_point <= 10000, "LuckyTreasure: reuire 0 < point <= 10000");
        require(0 < _slash_point && _slash_point <= 10000, "LuckyTreasure: reuire 0 < point <= 10000");

        fee_point = _slash_point;
        slash_point = _slash_point;

        emit NewPoint(_slash_point, _slash_point);
    }

    // call by admin
    function set_duration(uint256 duration) public onlyAdmin {
        require(duration >= 1 hours, "LuckyTreasure: require duration >= 1 hours");

        max_duration = duration;

        emit NewDuration(duration);
    }

    // call by creator
    function create(
        IERC20  treasure_token,
        uint256 treasure_balance,
        IERC20  ticket_token,
        uint256 ticket_balance,
        uint256 ticket_amount,
        bytes32 secret_hash
    ) public {
        require(ticket_amount > 0 && ticket_amount <= max_tickets, "LuckyTreasure: too many tickets");
        require(treasure_balance > 0 && ticket_balance > 0, "LuckyTreasure: zero balance");

        TreasureInfo storage t_info = treasure_infos[next_id];
        t_info.treasure_token = treasure_token;
        t_info.ticket_token = ticket_token;
        t_info.treasure_balance = treasure_balance;
        t_info.ticket_balance = ticket_balance;
        t_info.ticket_amount = ticket_amount;
        t_info.expired_time = block.timestamp + max_duration;
        t_info.creator = _msgSender();

        ParticipantInfo storage p_info = participant_infos[next_id];
        p_info.secret_hash = secret_hash;
        p_info.remain_tickets = ticket_amount;
        p_info.last_hash = bytes32(block.number);

        RefundInfo storage r_info = refund_infos[next_id];
        r_info.refund_time = block.timestamp + max_duration + max_duration;

        treasure_token.safeTransferFrom(
            address(_msgSender()),
            address(this),
            treasure_balance
        );

        emit NewTreasure(next_id);

        next_id = next_id + 1;
    }

    // call by participant
    function participate(
        uint256 id,
        uint256 tickets
    ) public {
        // 1. Update ParticipantInfo
        uint256 max_ticket_balance = update_participant_info(id, tickets);

        // 2. Update RefundInfo
        RefundInfo storage r_info = refund_infos[id];

        if (r_info.balances[_msgSender()] == 0) {
            r_info.remain_accounts += 1;
        }

        r_info.balances[_msgSender()] += max_ticket_balance;
    }

    function update_participant_info(
        uint256 id,
        uint256 tickets
    ) internal returns (uint256) {
        require(block.timestamp < treasure_infos[id].expired_time, "LuckyTreasure: has expired");
        require(tickets > 0, "LuckyTreasure: require tickets > 0");

        ParticipantInfo storage p_info = participant_infos[id];
        require(p_info.remain_tickets > 0, "LuckyTreasure: no remain ticket");

        uint256 max_ticket_amount = tickets;
        if (max_ticket_amount > p_info.remain_tickets) {
            max_ticket_amount = p_info.remain_tickets;
        }
        p_info.remain_tickets -= max_ticket_amount;

        uint256 token_balance = max_ticket_amount * treasure_infos[id].ticket_balance;
        require(token_balance >= treasure_infos[id].ticket_balance, "LuckyTreasure: token balance overflow");

        treasure_infos[id].ticket_token.safeTransferFrom(
            address(_msgSender()),
            address(this),
            token_balance
        );

        for (uint i = 0; i < max_ticket_amount; i++) {
            p_info.accounts.push(_msgSender());
        }

        // update last_hash
        p_info.last_hash = keccak256(abi.encodePacked(
                p_info.last_hash,
                _msgSender(),
                p_info.remain_tickets,
                block.timestamp
            ));

        emit Participate(id, _msgSender(), max_ticket_amount);

        return token_balance;
    }

    // call by creator or anyone who known the secret
    function open(
        uint256 id,
        bytes memory secret
    ) public {
        TreasureInfo storage t_info = treasure_infos[id];
        ParticipantInfo memory p_info = participant_infos[id];

        require(block.timestamp <= t_info.expired_time, "LuckyTreasure: has expired");
        require(t_info.winner == address(0), "LuckyTreasure: has opened");
        require(p_info.secret_hash == keccak256(bytes(secret)), "LuckyTreasure: invalid secret");
        require(p_info.remain_tickets == 0, "LuckyTreasure: remain tickets");

        address winner = draw_winner(id, p_info.last_hash, secret);

        uint256 ticket_balances = t_info.ticket_amount * t_info.ticket_balance;
        uint256 service_fee = calc_fee(ticket_balances);

        // transfer treasure token from this to winner
        t_info.treasure_token.safeTransfer(
            winner,
            t_info.treasure_balance
        );

        // transfer tickets token from this to owner
        t_info.ticket_token.safeTransfer(
            owner(),
            service_fee
        );

        // transfer tickets token from this to creator
        if (ticket_balances > service_fee) {
            t_info.ticket_token.safeTransfer(
                t_info.creator,
                (ticket_balances - service_fee)
            );
        }

        t_info.winner = winner;

        emit Winner(id, winner);

        delete refund_infos[id];
        delete participant_infos[id];
    }


    // draw lucky winners
    function draw_winner(uint256 id, bytes32 last_hash, bytes memory secret) internal view returns (address) {
        uint256 participants = participant_infos[id].accounts.length;

        require(participants > 0, "LuckyTreasure: no participant");

        uint256 random = uint256(keccak256(abi.encodePacked(last_hash, secret))) % participants;

        return participant_infos[id].accounts[random];
    }

    function calc_fee(uint256 total_balance) public view returns (uint256) {
        return total_balance * fee_point / 10000;
    }

    function calc_slash(uint256 total_balance) public view returns (uint256) {
        return total_balance * slash_point / 10000;
    }

    // call by anyone (too expensive)
    function refund_all(uint256 id) public {
        TreasureInfo storage t_info = treasure_infos[id];
        ParticipantInfo memory p_info = participant_infos[id];
        uint256 refund_time = refund_infos[id].refund_time;

        require(block.timestamp > t_info.expired_time && block.timestamp <= refund_time, "LuckyTreasure: ban refund all");
        require(t_info.winner == address(0), "LuckyTreasure: has winner");
        require(!t_info.refunded_all, "LuckyTreasure: has refunded all");

        // refund ticket token to participant
        // max_transfter = max_tickets = 1000
        for (uint i = 0; i < p_info.accounts.length; i++ ) {
            if (p_info.accounts[i] != address(0)) {
                t_info.ticket_token.safeTransfer(p_info.accounts[i], t_info.ticket_balance);
            }
        }

        // slash treasure token to owner
        uint256 slash_balance = 0;
        if (t_info.winner == address(0) && p_info.remain_tickets == 0) {
            slash_balance = calc_slash(t_info.treasure_balance);

            t_info.treasure_token.safeTransfer(
                owner(),
                slash_balance
            );

            emit Slash(id, slash_balance);
        }

        // refund treasure token to creator
        t_info.treasure_token.safeTransfer(
            t_info.creator,
            (t_info.treasure_balance - slash_balance)
        );

        t_info.refunded_all = true;

        emit RefundAll(id);

        delete refund_infos[id];
        delete participant_infos[id];
    }

    // call by participant
    function refund(uint256 id) public {
        TreasureInfo memory t_info = treasure_infos[id];
        ParticipantInfo memory p_info = participant_infos[id];
        RefundInfo storage r_info = refund_infos[id];

        require(block.timestamp > r_info.refund_time, "LuckyTreasure: ban refund");
        require(r_info.remain_accounts > 0 && p_info.accounts.length > 0, "LuckyTreasure: no remain account");
        require(t_info.winner == address(0), "LuckyTreasure: has winner");
        require(!t_info.refunded_all, "LuckyTreasure: has refunded all");

        uint256 total_ticket_balance = 0;
        for (uint i = 0; i < p_info.accounts.length; i++) {
            total_ticket_balance += t_info.ticket_balance;
        }

        uint256 token_balance = r_info.balances[_msgSender()];
        if (token_balance > 0) {
            t_info.ticket_token.safeTransfer(_msgSender(), token_balance);

            uint256 treasure_balance = token_balance * t_info.treasure_balance / total_ticket_balance;
            if (treasure_balance > 0) {
                t_info.treasure_token.safeTransfer(_msgSender(), treasure_balance);
            }

            r_info.remain_accounts -= 1;
            // delete has refunded
            delete r_info.balances[_msgSender()];
        }

        if (r_info.remain_accounts == 0) {
            delete refund_infos[id];
            delete participant_infos[id];
        }
    }

    // call by anyone
    // calculate odds: total_tickets / holding_tickets
    function get_pending_tickets(uint256 id, address participant) public view returns (uint256, uint256) {
        uint256 holding_tickets = 0;
        for (uint i = 0; i < participant_infos[id].accounts.length; i++) {
            if (participant == participant_infos[id].accounts[i]) {
                holding_tickets += 1;
            }
        }

        return (holding_tickets, treasure_infos[id].ticket_amount);
    }
}
