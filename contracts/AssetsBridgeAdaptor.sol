// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

/**
 * @dev Interface of the SherpaX AssetsBridge
 */
interface IAssetsBridge {
    /*
     * @dev mint the token to account for assets bridge admin.
     * @param account The receiver of token.
     * @param amount The amount of token.
     */
    function mint_into(address account, uint256 amount) external returns (bool);

    /*
     * @dev burn the token from account for assets bridge admin.
     * @param account The owner of token.
     * @param amount The amount of token.
     */
    function burn_from(address account, uint256 amount) external returns (bool);
}

library AssetsBridgeLibrary {
    address public constant admin = 0x1111111111111111111111111111111111111111;

    function get_admin() public pure returns(address){
        return admin;
    }
}
