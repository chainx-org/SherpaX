//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./AssetsBridgeErc20_OnlyOwner.sol";

contract StakingContract is Ownable{

    AssetsBridgeErc20 private MINI_Erc20;
    AssetsBridgeErc20 private Staking_Erc20;
    uint256 private index;
    // 7 days
    //uint256 constant ACTIVE_DURATION = 604800;
    //uint256 constant INACTIVE_DURATION = 604800;
    //3 mins
    uint256 constant ACTIVE_DURATION = 180;
    uint256 constant INACTIVE_DURATION = 180;

    struct Info{
        uint256 staking_balance;
        bool is_claimed;
    }
    struct Pool{
        uint256 total_mini;
        uint256 create_timpstamp;
        uint256 total_balance;
    }
    mapping(uint256 => mapping(address=> Info)) public pool_staking_info;
    mapping(uint256 => Pool) public index_pool;

    event CreateMINIPool(address _caller, uint256 _total_mini);
    event Staking(address _caller, uint256 _index,uint256 _amount);
    event Claim(address _caller, uint256 _index,uint256 _share);

    constructor(address _mini, address _erc20) {
        MINI_Erc20 = AssetsBridgeErc20(_mini);
        Staking_Erc20 = AssetsBridgeErc20(_erc20);
    }
    modifier ActiveIndexRequire(uint256 _index) {
        require(index_pool[_index].create_timpstamp != 0,"Pool does not exist");
        _;
    }
    function create_mini_pool(uint256 _total_mini) public onlyOwner {
        bool result = MINI_Erc20.transferFrom(msg.sender,address(this),_total_mini);
        if (result) {
            index_pool[index++] = Pool(_total_mini,block.timestamp,0);
            emit CreateMINIPool(msg.sender,_total_mini);
        }
    }
    function staking(uint256 _index,uint256 _amount) public ActiveIndexRequire(_index){
        uint256 create_timpstamp = index_pool[_index].create_timpstamp;
        require(block.timestamp - create_timpstamp <= ACTIVE_DURATION,"ACTIVE_DURATION timeout");
        bool result = Staking_Erc20.transferFrom(msg.sender,address(this),_amount);
        if (result) {
            index_pool[_index].total_balance += _amount;
            Info storage info = pool_staking_info[_index][msg.sender];
            info.staking_balance+=_amount;
            emit Staking(msg.sender,_index,_amount);
        }
    }
    function claim(uint256 _index) public ActiveIndexRequire(_index){
        uint256 create_timpstamp = index_pool[_index].create_timpstamp;
        require(block.timestamp - create_timpstamp >= ACTIVE_DURATION + INACTIVE_DURATION,"Claim didn't start");
        (uint256 staking_balance,uint256 share,bool is_claimed) = get_share(_index);
        if(!is_claimed){
            MINI_Erc20.transfer(msg.sender,share);
            Staking_Erc20.transfer(msg.sender,staking_balance);
            emit Claim(msg.sender,_index,share);
        }
    }
    function get_share(uint256 _index) public view ActiveIndexRequire(_index) returns(uint256,uint256, bool) {
        Info memory info = pool_staking_info[_index][msg.sender];
        Pool memory pool = index_pool[_index];
        uint256 share = info.staking_balance * pool.total_mini / pool.total_balance;
        return (info.staking_balance,share,info.is_claimed);
    }
    function get_mini_address() public view returns(address){
        return address(MINI_Erc20);
    }
    function get_erc20_address() public view returns(address){
        return address(Staking_Erc20);
    }
    function get_pool_index() public view returns(uint256){
        return index;
    }

}