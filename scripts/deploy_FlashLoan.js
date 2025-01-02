const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Load configuration for flashloan
const configPath = path.join(__dirname, "../config/flashloan_config.json");
const abiDirPath = path.join(__dirname, "../abi");
const abiPath = path.join(abiDirPath, "flashloan_abi.json");

function validateConfig(config) {
    const { lendingPoolAddress, wethAddress } = config;

    if (!lendingPoolAddress) {
        throw new Error("Lending pool address is missing in the flashloan configuration.");
    }

    if (!wethAddress) {
        throw new Error("WETH address is missing in the flashloan configuration.");
    }
}

function saveAbiAndConfig(flashloanAddress, abi) {
    console.log("Saving new contract address and ABI...");

    // Save the contract address to config
    const updatedConfig = { ...config, flashloanContractAddress: flashloanAddress };
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
        console.log(`Deploying FlashLoan contract...`);

        const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));
        validateConfig(config);

        // Get the FlashLoan contract factory and deploy the contract
        const FlashLoan = await hre.ethers.getContractFactory("FlashLoan");
        const flashLoan = await FlashLoan.deploy(config.lendingPoolAddress, config.wethAddress);
        await flashLoan.deployed();

        console.log(`FlashLoan contract deployed at: ${flashLoan.address}`);

        // Update ABI and config files
        const abi = JSON.stringify(artifacts.readArtifactSync("FlashLoan").abi, null, 2);
        saveAbiAndConfig(flashLoan.address, abi);

    } catch (error) {
        console.error("Error deploying FlashLoan contract:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Unhandled error:", error.message);
        process.exit(1);
    });

