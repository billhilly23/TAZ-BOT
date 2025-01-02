const hre = require("hardhat");
const fs = require("fs");
const path = require("path");
const { updateABI, updateConfig } = require("./deploy_helpers");

async function main() {
    const contractName = "X";  // Replace 'X' with the actual contract name
    console.log(`Starting deployment of ${contractName} contract...`);

    // Load configuration for the contract
    const configPath = path.join(__dirname, "../config/X_config.json");  // Adjust the config file name accordingly
    const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));

    // Fetch required addresses from the config file
    const { uniswapRouter, sushiswapRouter } = config;

    if (!uniswapRouter || !sushiswapRouter) {
        console.error("Uniswap and Sushiswap router addresses must be set in the configuration file.");
        process.exit(1);
    }

    try {
        // Compile and deploy the contract
        const ContractFactory = await hre.ethers.getContractFactory(contractName);
        const contractInstance = await ContractFactory.deploy(uniswapRouter, sushiswapRouter);
        await contractInstance.deployed();

        console.log(`${contractName} contract deployed at: ${contractInstance.address}`);

        // Update ABI and config
        updateABI(contractName, contractInstance);
        updateConfig(contractName, contractInstance.address);

        console.log(`${contractName} ABI and config updated successfully.`);
    } catch (error) {
        console.error(`Error during the deployment of ${contractName}:`, error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(`Unhandled error during ${contractName} deployment:`, error.message);
        process.exit(1);
    });

