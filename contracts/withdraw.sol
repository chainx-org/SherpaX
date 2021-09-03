pragma solidity ^0.7.0;

contract WithdrawPrecompile {
     event Withdraw(address indexed from, bytes32 indexed to, uint256 value);

     // evm account transfer ksx to substrate account
     // @to is substrate account public key
     // @value is amount of balance
     function withdraw(
        bytes32 to,
        uint256 value
    ) public returns (bool) {
        bytes memory input = abi.encodePacked(msg.sender, to, value);

        // Dynamic arrays will add the array size to the front of the array, so need extra 32 bytes.
        uint len = input.length;

        assembly {
            if iszero(
                // from(20 bytes) + to(32 bytes) + value(32 bytes)
                staticcall(gas(), 0x401, add(input, 0x20), len, 0x00, 0x00)
            ) {
                revert(0, 0)
            }
        }

        emit Withdraw(msg.sender, to, value);

        return true;
    }
}
