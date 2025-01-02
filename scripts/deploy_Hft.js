const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Load configuration for HFT
const configPath = path.join(__dirname, "../config/hft_config.json");
const abiDirPath = path.join(__dirname, "../abi");
const abiFilePath = path.join(abiDirPath, "hft_abi.json");

function validateConfig(config) {
    const { uniswapRouter, sushiswapRouter } = config;

    if (!uniswapRouter || !sushiswapRouter) {
        throw new Error("Both Uniswap and Sushiswap router addresses must be set in the configuration file.");
    }
}

function saveAbiAndConfig(hftAddress, abi) {
    console.log("Saving new contract address and ABI...");

    // Update the config file with the new contract address
    const updatedConfig = { ...config, hftContractAddress: hftAddress };
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
        console.log("Loading configuration for HFT contract...");

        const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));
        validateConfig(config);

        console.log(`Deploying HFT contract...`);

        // Get the HFT contract factory and deploy it
        const HFT = await hre.ethers.getContractFactory("HFT");
        const hft = await HFT.deploy(config.uniswapRouter, config.sushiswapRouter);
        await hft.deployed();

        console.log(`HFT contract deployed at: ${hft.address}`);

        // Update ABI and configuration files
        const abi = JSON.stringify(artifacts.readArtifactSync("HFT").abi, null, 2);
        saveAbiAndConfig(hft.address, abi);

    } catch (error) {
        console.error("Error deploying HFT contract:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Unhandled error:", error.message);
        process.exit(1);
    });

