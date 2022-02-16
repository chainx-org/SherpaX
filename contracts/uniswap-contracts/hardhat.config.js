/**
 * @type import('hardhat/config').HardhatUserConfig
 */

require('@nomiclabs/hardhat-ethers');

// Change private keys accordingly - ONLY FOR DEMOSTRATION PURPOSES - PLEASE STORE PRIVATE KEYS IN A SAFE PLACE
// Export your private key as
// export PRIVKEY=0x.....
const privateKey = process.env.PRIVKEY;
const privateKeyDev =
   '0x99b3c12287537e38c90a9219d4cb074a89a16e9cdb20bf85728ebd97c343e342';

module.exports = {
   defaultNetwork: 'hardhat',

   networks: {
      hardhat: {},
      mainnet: {
         url: 'https://minichain-mainnet.coming.chat/rpc',
         accounts: [privateKey],
         network_id: '1506',
         chainId: 1506,
      },
      testnet: {
         url: 'https://sherpax-testnet.chainx.org/rpc',
         accounts: [privateKey],
         network_id: '1506',
         chainId: 1506,
      },
      dev: {
         url: 'http://8.136.153.42:8546',
         accounts: [privateKeyDev],
         network_id: '1506',
         chainId: 1506,
      },
   },
   solidity: {
      compilers: [
         {
            version: '0.5.16',
            settings: {
               optimizer: {
                  enabled: true,
                  runs: 200,
               },
            },
            evmVersion: "istanbul"
         },
         {
            version: '0.6.6',
            settings: {
               optimizer: {
                  enabled: true,
                  runs: 200,
               },
            },
         },
      ],
   },
   paths: {
      sources: './contracts',
      cache: './cache',
      artifacts: './artifacts',
   },
   mocha: {
      timeout: 20000,
   },
};
