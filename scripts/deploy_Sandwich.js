const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Load configuration for sandwich
const configPath = path.join(__dirname, "../config/sandwich_config.json");
const abiDirPath = path.join(__dirname, "../abi");
const abiFilePath = path.join(abiDirPath, "sandwich_abi.json");

function validateConfig(config) {
    const { uniswapRouter } = config;

    if (!uniswapRouter) {
        throw new Error("Uniswap router address must be set in the configuration file.");
    }
}

function saveAbiAndConfig(sandwichAddress, abi) {
    console.log("Saving new contract address and ABI...");

    // Update the config file with the new contract address
    const updatedConfig = { ...config, sandwichContractAddress: sandwichAddress };
    fs.writeFileSync(configPath, JSON.stringify(updatedConfig, null, 2));

    // Ensure the ABI directory exists
    if (!fs.existsSync(abiDirPath)) {
        console.log(`ABI directory doesn't exist, creating: ${abiDirPath}`);
        fs.mkdirSync(abiDirPath);
    }

    // Write the contract ABI
    fs.writeFileSync(abiFilePath, abi);
    console.log("ABI and config updated successfully.");
}

async function main() {
    try {
        console.log("Loading configuration for Sandwich contract...");

        const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));
        validateConfig(config);

        console.log(`Deploying Sandwich contract...`);

        // Get the Sandwich contract factory and deploy it
        const Sandwich = await hre.ethers.getContractFactory("Sandwich");
        const sandwich = await Sandwich.deploy(config.uniswapRouter);
        await sandwich.deployed();

        console.log(`Sandwich contract deployed at: ${sandwich.address}`);

        // Update ABI and configuration files
        const abi = JSON.stringify(artifacts.readArtifactSync("Sandwich").abi, null, 2);
        saveAbiAndConfig(sandwich.address, abi);

    } catch (error) {
        console.error("Error deploying Sandwich contract:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Unhandled error:", error.message);
        process.exit(1);
    });

