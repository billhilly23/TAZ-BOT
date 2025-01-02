const express = require('express');
const { web3 } = require('../web3');  // Assuming web3 is initialized globally
const Arbitrage = require('../abi/arbitrage_abi.json');  // Arbitrage ABI
const config = require('../config/arbitrage_config.json');  // Arbitrage config
const router = express.Router();

router.post('/', async (req, res) => {
    try {
        const arbitrageContract = new web3.eth.Contract(Arbitrage, config.arbitrage_contract_address);

        const tradeSize = req.body.tradeSize;
        const gasLimit = req.body.gasFee;
        const slippage = req.body.slippage;

        const tx = await arbitrageContract.methods.executeArbitrage(tradeSize, gasLimit, slippage)
            .send({
                from: process.env.WALLET_ADDRESS,
                gas: gasLimit
            });

        res.json({ message: 'Arbitrage transaction executed', txHash: tx.transactionHash });
    } catch (error) {
        console.error(`Error executing Arbitrage transaction: ${error.message}`);
        res.status(500).json({ message: 'Arbitrage transaction failed', error: error.message });
    }
});

module.exports = router;

