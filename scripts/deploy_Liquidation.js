const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Load configuration for liquidation
const configPath = path.join(__dirname, "../config/liquidation_config.json");
const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));

async function main() {
    const contractName = "Liquidation";
    console.log(`Deploying ${contractName} contract...`);

    // Fetch addresses and necessary parameters from the config file
    const { 
        aavePoolAddress, 
        compoundComptrollerAddress,
        fromAccount,
        ctokenBorrowed, 
        ctokenCollateral 
    } = config;

    // Validate necessary addresses and parameters
    if (!aavePoolAddress || !compoundComptrollerAddress || !ctokenBorrowed || !ctokenCollateral) {
        console.error("Missing required addresses or parameters in the config file.");
        process.exit(1);
    }

    // Deploy the Liquidation contract
    const Liquidation = await hre.ethers.getContractFactory(contractName);
    const liquidation = await Liquidation.deploy(
        aavePoolAddress,
        compoundComptrollerAddress,
        fromAccount,
        ctokenBorrowed,
        ctokenCollateral
    );
    
    await liquidation.deployed();

    console.log(`${contractName} contract deployed at: ${liquidation.address}`);

    // Update config with new contract address
    config.liquidationContractAddress = liquidation.address;
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));

    // Update the ABI file
    const abiPath = path.join(__dirname, "../abi/liquidation_abi.json");
    const abi = JSON.stringify(artifacts.readArtifactSync(contractName).abi, null, 2);
    fs.writeFileSync(abiPath, abi);

    console.log(`${contractName} ABI and config updated.`);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(`Error deploying ${contractName}:`, error);
        process.exit(1);
    });

