/**
 * @type import('hardhat/config').HardhatUserConfig
 */

require('@nomiclabs/hardhat-ethers');

// Change private keys accordingly - ONLY FOR DEMOSTRATION PURPOSES - PLEASE STORE PRIVATE KEYS IN A SAFE PLACE
// Export your private key as
// export PRIVKEY=0x.....
const privateKey = process.env.PRIVKEY;

module.exports = {
   defaultNetwork: 'hardhat',

   networks: {
      hardhat: {},
      mainnet: {
         url: 'https://mainnet.sherpax.io/rpc',
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
         url: 'http://127.0.0.1:8546',
         accounts: [privateKey],
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
