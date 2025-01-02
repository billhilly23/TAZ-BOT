const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Load configuration for frontrunning
const configPath = path.join(__dirname, "../config/front_running_config.json");
const abiDirPath = path.join(__dirname, "../abi");
const abiPath = path.join(abiDirPath, "frontrunning_abi.json");

function validateConfig(config) {
    const { uniswapRouter } = config;

    if (!uniswapRouter) {
        throw new Error("Uniswap router address is missing in the configuration file.");
    }
}

function saveAbiAndConfig(frontRunningAddress, abi) {
    console.log("Saving new contract address and ABI...");

    // Update the config file with the new contract address
    const updatedConfig = { ...config, frontRunningContractAddress: frontRunningAddress };
    fs.writeFileSync(configPath, JSON.stringify(updatedConfig, null, 2));

    // Ensure the ABI directory exists
    if (!fs.existsSync(abiDirPath)) {
        fs.mkdirSync(abiDirPath);
    }

    // Save the contract ABI
    fs.writeFileSync(abiPath, abi);
    console.log("ABI and config updated successfully.");
}

async function main() {
    try {
        console.log("Loading configuration for FrontRunning contract...");

        const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));
        validateConfig(config);

        console.log(`Deploying FrontRunning contract...`);

        // Get the contract factory and deploy the contract
        const FrontRunning = await hre.ethers.getContractFactory("FrontRunning");
        const frontRunning = await FrontRunning.deploy(config.uniswapRouter);
        await frontRunning.deployed();

        console.log(`FrontRunning contract deployed at: ${frontRunning.address}`);

        // Update the ABI and configuration files
        const abi = JSON.stringify(artifacts.readArtifactSync("FrontRunning").abi, null, 2);
        saveAbiAndConfig(frontRunning.address, abi);

    } catch (error) {
        console.error("Error deploying FrontRunning contract:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Unhandled error:", error.message);
        process.exit(1);
    });

