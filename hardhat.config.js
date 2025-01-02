require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-etherscan");
require("hardhat-gas-reporter");
require("dotenv").config();

// Ensure required environment variables are set
if (!process.env.INFURA_PROJECT_ID) {
  console.error("INFURA_PROJECT_ID is missing in the .env file");
  process.exit(1);
}

if (!process.env.DEPLOYER_PRIVATE_KEY) {
  console.error("DEPLOYER_PRIVATE_KEY is missing in the .env file");
  process.exit(1);
}

module.exports = {
  // Network configurations
  networks: {
    // Hardhat local network
    hardhat: {
      chainId: 1337,
    },
    // Rinkeby testnet
    rinkeby: {
      url: `https://rinkeby.infura.io/v3/${process.env.INFURA_PROJECT_ID}`, // Uses Infura
      accounts: [`0x${process.env.DEPLOYER_PRIVATE_KEY}`],  // Deployer's private key
    },
    // Ethereum mainnet
    mainnet: {
      url: `https://mainnet.infura.io/v3/${process.env.INFURA_PROJECT_ID}`, // Uses Infura
      accounts: [`0x${process.env.DEPLOYER_PRIVATE_KEY}`],  // Deployer's private key
    },
    // Add other networks (Kovan, Goerli, etc.) if needed
  },

  // Solidity compiler settings
  solidity: {
    version: "0.8.4",  // Set Solidity version for your contracts
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,  // Optimization settings to reduce gas costs
      },
    },
  },

  // Path settings (optional)
  paths: {
    sources: "./contracts",            // Default: './contracts'
    tests: "./test",                   // Default: './test'
    cache: "./cache",                  // Default: './cache'
    artifacts: "./artifacts",          // Default: './artifacts'
  },

  // Etherscan API key (optional)
  etherscan: {
    apiKey: process.env.ETHERSCAN_API_KEY,  // Used to verify contracts on Etherscan
  },

  // Gas reporter configuration
  gasReporter: {
    enabled: process.env.REPORT_GAS !== undefined,  // Enable gas reporting if REPORT_GAS is set in .env
    currency: 'USD',  // Currency for gas cost reports
    coinmarketcap: process.env.COINMARKETCAP_API_KEY,  // Required if you want to fetch the latest gas prices
  },
};

