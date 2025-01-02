use serde_json::Value;
use std::fs;
use web3::types::{U256, Address};
use web3::contract::Options;
use web3::contract::Contract;
use log::{error, info};
use tokio::task;
use thiserror::Error;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

// Load the HFT configuration
fn load_hft_config() -> Value {
    let config_path = "config/hft_config.json";
    let config_data = fs::read_to_string(config_path)
        .expect("Unable to read HFT config file");
    let config: Value = serde_json::from_str(&config_data)
        .expect("Unable to parse HFT config file");
    config
}

// Continuous Monitoring: Monitor price movements on DEXs
pub async fn monitor_price_movements(
    web3: Arc<web3::Web3<web3::transports::Http>>,
    config: &Value,
    check_interval: u64
) -> Result<(), HFTError> {
    let asset: Address = config["asset"].as_str().unwrap().parse().expect("Invalid asset address");
    let uniswap_router_contract = Contract::from_json(
        web3.eth(),
        str_to_address(config["uniswap_router_address"].as_str().unwrap()),
        include_bytes!("abi/uniswap_router_abi.json"),
    )?;

    loop {
        let price = get_asset_price(web3.clone(), uniswap_router_contract.clone(), asset).await?;
        info!("Current price: {:?}", price);

        // Logic to determine if this is a short-term trading opportunity
        if should_trade(price) {
            info!("Trading opportunity detected!");
            execute_hft(web3.clone()).await?;
        }

        // Monitor at intervals
        sleep(Duration::from_secs(check_interval)).await;
    }
}

// Get asset price from Uniswap or another DEX
pub async fn get_asset_price(
    web3: Arc<web3::Web3<web3::transports::Http>>,
    uniswap_router_contract: Contract<web3::transports::Http>,
    asset: Address
) -> Result<U256, HFTError> {
    let price: U256 = uniswap_router_contract
        .query("getAmountsOut", (U256::from(1u64), vec![asset]), None, Options::default(), None)
        .await
        .map_err(HFTError::ContractError)?;

    Ok(price)
}

// Logic to determine if a trade should be executed based on price movement
fn should_trade(current_price: U256) -> bool {
    // Example: Simple logic, you could improve with technical indicators or thresholds
    let target_price = U256::from(3000); // Example target price
    current_price < target_price
}

// Quick Execution: Execute HFT logic with flash loans (with parallel execution)
pub async fn execute_hft(
    web3: Arc<web3::Web3<web3::transports::Http>>
) -> Result<(), HFTError> {
    // Load HFT configuration
    let config = load_hft_config();
    let asset: Address = config["asset"].as_str().unwrap().parse().expect("Invalid address");
    let module: String = config["module"].as_str().unwrap().to_string();
    let expected_profit = U256::from_dec_str(config["expected_profit"].as_str().unwrap()).expect("Invalid profit amount");
    let gas_fee = U256::from_dec_str(config["gas_fee"].as_str().unwrap()).expect("Invalid gas fee");
    let slippage: f64 = config["slippage"].as_f64().expect("Invalid slippage");

    // Calculate dynamic loan amount
    let flashloan_amount = calculate_dynamic_loan_amount(expected_profit, gas_fee, slippage);

    let web3_clone = web3.clone();
    task::spawn(async move {
        info!("Starting HFT module: {} with flash loan amount: {}", module, flashloan_amount);

        // Execute flash loan for HFT
        match request_flash_loan(&web3_clone, asset, flashloan_amount).await {
            Ok(_) => {
                info!("Flash loan successful for HFT module");

                // ** HFT Strategy: Executing a Trade based on market conditions **
                match execute_trade(web3_clone.clone(), asset).await {
                    Ok(_) => info!("HFT strategy executed successfully"),
                    Err(e) => error!("HFT strategy execution failed: {}", e),
                }
            },
            Err(e) => error!("Flash loan failed for HFT module: {}", e),
        }
    })
    .await
    .map_err(|e| HFTError::JoinError(e))?;

    Ok(())
}

// HFT Trading Logic: Execute the actual trade after flash loan is received
pub async fn execute_trade(
    web3: Arc<web3::Web3<web3::transports::Http>>,
    asset: Address
) -> Result<(), HFTError> {
    let uniswap_router_contract = Contract::from_json(
        web3.eth(),
        "UNISWAP_ROUTER_ADDRESS".parse().unwrap(),
        include_bytes!("abi/uniswap_router_abi.json"),
    )?;

    let amount_in: U256 = U256::from(1000);  // Example amount
    let gas_limit = U256::from(300000);  // Example gas limit
    let path = vec![asset, "0xTOKEN_B_ADDRESS".parse().unwrap()];  // Example trade path

    let result = uniswap_router_contract
        .call("swapExactTokensForTokens", (amount_in, U256::from(1), path, "YOUR_ADDRESS".parse().unwrap(), U256::from(3000000000u64)), Address::zero(), Options::default())
        .await;

    match result {
        Ok(tx) => {
            info!("HFT trade executed successfully: {:?}", tx);
            Ok(())
        }
        Err(e) => {
            error!("HFT trade execution failed: {:?}", e);
            Err(HFTError::ContractError(e))
        }
    }
}

// Flash Loan Execution Logic
pub async fn request_flash_loan(
    web3: &web3::Web3<web3::transports::Http>,
    asset: Address,
    amount: U256
) -> Result<(), HFTError> {
    let aave_flashloan_contract = Contract::from_json(
        web3.eth(),
        "AAVE_FLASHLOAN_CONTRACT_ADDRESS".parse().unwrap(),
        include_bytes!("abi/aave_flashloan_abi.json"),
    )?;

    let params = (
        vec![asset],
        vec![amount],
        vec![0],
        "YOUR_ADDRESS".parse().unwrap(),
        vec![0u8],
    );

    let tx = aave_flashloan_contract
        .call("flashLoan", params, Address::zero(), Options::default())
        .await?;

    info!("Flash loan executed: {:?}", tx);
    Ok(())
}

// Custom error type for HFT
#[derive(Error, Debug)]
pub enum HFTError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("Contract error: {0}")]
    ContractError(#[from] web3::contract::Error),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

// Implement conversion for HFTError to Web3 error
impl From<HFTError> for web3::Error {
    fn from(error: HFTError) -> Self {
        web3::Error::Decoder(format!("{:?}", error))
    }
}

// Helper function to convert string to Address
fn str_to_address(address: &str) -> Address {
    Address::from_str(address).unwrap()
}

