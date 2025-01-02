const express = require('express');
const { spawn } = require('child_process');
const router = express.Router();

router.post('/', (req, res) => {
    try {
        const child = spawn('npx', ['hardhat', 'run', './scripts/deploy_Liquidation.js']);

        child.stdout.on('data', (data) => {
            console.log(`stdout: ${data}`);
        });

        child.stderr.on('data', (data) => {
            console.error(`stderr: ${data}`);
        });

        child.on('error', (error) => {
            console.error(`Error executing Liquidation script: ${error.message}`);
            res.status(500).json({ message: 'Liquidation script failed', error: error.message });
        });

        child.on('close', (code) => {
            if (code === 0) {
                res.json({ message: 'Liquidation script executed successfully' });
            } else {
                res.status(500).json({ message: `Liquidation script exited with code ${code}` });
            }
        });
    } catch (error) {
        res.status(500).json({ message: 'Unexpected error occurred', error: error.message });
    }
});

module.exports = router;

