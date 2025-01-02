const Web3 = require('web3');
require('dotenv').config();  // Load environment variables from .env file

// Initialize Web3 with Infura
const web3 = new Web3(new Web3.providers.HttpProvider(`https://${process.env.NETWORK}.infura.io/v3/${process.env.INFURA_PROJECT_ID}`));

// Unlock the wallet using the private key
const account = web3.eth.accounts.privateKeyToAccount(process.env.WALLET_PRIVATE_KEY);

// Add the account to the wallet
web3.eth.accounts.wallet.add(account);

// Export the configured web3 instance for use in other modules
module.exports = { web3 };

