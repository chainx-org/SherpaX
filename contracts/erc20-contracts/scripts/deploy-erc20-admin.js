const { ethers } = require('hardhat');

// Deploy function
async function deploy() {
   [account] = await ethers.getSigners();
   deployerAddress = account.address;
   console.log(`Deploying contracts using ${deployerAddress}`);
    //Deploy SBTC (needed for Interface)
    const SBTC = await ethers.getContractFactory('contracts/AssetsBridgeErc20_OnlyAdmin.sol:AssetsBridgeErc20');
    const SBTCInstance = await SBTC.deploy("SBTC","SBTC",18);
    await SBTCInstance.deployed();
    console.log(`SBTC deployed to : ${SBTCInstance.address}`);

    //Deploy KSM (needed for Interface)
    const KSM = await ethers.getContractFactory('contracts/AssetsBridgeErc20_OnlyAdmin.sol:AssetsBridgeErc20');
    const KSMInstance = await SBTC.deploy("KSM","KSM",18);
    await KSMInstance.deployed();
    console.log(`KSM deployed to : ${SBTCInstance.address}`);
}

deploy()
   .then(() => process.exit(0))
   .catch((error) => {
      console.error(error);
      process.exit(1);
   });
