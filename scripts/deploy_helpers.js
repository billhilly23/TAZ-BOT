const fs = require('fs');
const path = require('path');

// Function to update the ABI file
const updateABI = (contractName, contractInstance) => {
    try {
        console.log(`Preparing to update ABI for ${contractName}...`);

        const abiDir = path.join(__dirname, "../abi");
        const abiFilePath = path.join(abiDir, `${contractName.toLowerCase()}_abi.json`);

        // Validate contract instance
        if (!contractInstance || !contractInstance.interface) {
            throw new Error(`Invalid contract instance for ${contractName}`);
        }

        // Read the ABI from the deployed contract
        const abi = JSON.stringify(contractInstance.interface.format('json'), null, 2);

        // Ensure the ABI directory exists
        if (!fs.existsSync(abiDir)) {
            console.log(`ABI directory doesn't exist, creating: ${abiDir}`);
            fs.mkdirSync(abiDir);
        }

        // Write the ABI to the file
        fs.writeFileSync(abiFilePath, abi);
        console.log(`${contractName} ABI successfully updated at ${abiFilePath}`);
    } catch (error) {
        console.error(`Failed to update ABI for ${contractName}:`, error.message);
    }
};

// Function to update the contract address in the config file
const updateConfig = (contractName, contractAddress) => {
    try {
        console.log(`Preparing to update config for ${contractName}...`);

        const configPath = path.join(__dirname, "../config", `${contractName.toLowerCase()}_config.json`);

        // Validate contract address
        if (!contractAddress || !/^0x[a-fA-F0-9]{40}$/.test(contractAddress)) {
            throw new Error(`Invalid contract address for ${contractName}`);
        }

        // Read the current config file
        const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));

        // Update the contract address in the config
        config[`${contractName.toLowerCase()}ContractAddress`] = contractAddress;

        // Write the updated config back to the file
        fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
        console.log(`${contractName} address successfully updated in ${configPath}`);
    } catch (error) {
        console.error(`Failed to update config for ${contractName}:`, error.message);
    }
};

module.exports = {
    updateABI,
    updateConfig
};

