const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Load configuration for arbitrage
const configPath = path.join(__dirname, "../config/arbitrage_config.json");
const abiDirPath = path.join(__dirname, "../abi");
const abiPath = path.join(abiDirPath, "arbitrage_abi.json");

function validateConfig(config) {
    const { uniswapRouter, sushiswapRouter } = config;
    
    if (!uniswapRouter) {
        throw new Error("Uniswap router address is missing in the config.");
    }
    
    if (!sushiswapRouter) {
        throw new Error("Sushiswap router address is missing in the config.");
    }
}

function saveAbiAndConfig(arbitrageAddress, abi) {
    console.log("Saving new contract address and ABI...");
    
    // Save the contract address to config
    const updatedConfig = { ...config, arbitrageContractAddress: arbitrageAddress };
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
        const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));

        // Validate that the necessary addresses are in the config
        validateConfig(config);

        console.log("Deploying Arbitrage contract...");

        // Get contract factory and deploy
        const Arbitrage = await hre.ethers.getContractFactory("Arbitrage");
        const arbitrage = await Arbitrage.deploy(config.uniswapRouter, config.sushiswapRouter);
        await arbitrage.deployed();

        console.log("Arbitrage contract deployed to:", arbitrage.address);

        // Update ABI and config files
        const abi = JSON.stringify(artifacts.readArtifactSync("Arbitrage").abi, null, 2);
        saveAbiAndConfig(arbitrage.address, abi);

    } catch (error) {
        console.error("Error deploying contract:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Unhandled error:", error.message);
        process.exit(1);
    });

