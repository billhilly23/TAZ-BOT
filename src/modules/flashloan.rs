use serde_json::Value;
use std::fs;
use web3::types::{U256, Address};
use web3::contract::Contract;
use log::{error, info};
use thiserror::Error;
use tokio::time::{sleep, Duration};
use web3::transports::Http;
use web3::contract::Options;
use std::str::FromStr;

// Load flashloan config
fn load_flashloan_config() -> Value {
    let config_path = "config/flashloan_config.json";
    let config_data = std::fs::read_to_string(config_path).expect("Unable to read flashloan config file");
    serde_json::from_str(&config_data).expect("Unable to parse flashloan config file")
}

// Convert string to Address
fn str_to_address(address: &str) -> Address {
    Address::from_str(address).unwrap()
}

// Dynamic loan calculation for flashloan opportunities
pub fn calculate_dynamic_loan_amount(expected_profit: U256, gas_fee: U256, slippage: f64) -> U256 {
    let slippage_factor = 1.0 - slippage;
    let max_loan_amount = (expected_profit.low_u64() as f64 * slippage_factor) as u64;
    U256::from(max_loan_amount).saturating_sub(gas_fee)
}

// Profitability Tracking for flashloans
pub fn is_profitable(profit: U256, gas_fees: U256) -> bool {
    profit > gas_fees
}

// Monitor liquidity pools for flashloan opportunities
pub async fn scan_for_flashloan_opportunities(
    web3: &web3::Web3<Http>,
    lending_pool: Address,
    check_interval: u64,
) {
    loop {
        // Retrieve liquidity data from the pool
        if let Ok(available_liquidity) = get_liquidity_data(web3, lending_pool).await {
            if available_liquidity > U256::from(1000000000000000000u64) {  // Example: 1 ETH liquidity
                info!("Flashloan opportunity detected with sufficient liquidity");
                // Trigger the flashloan execution if profitable
                let loan_amount = calculate_dynamic_loan_amount(U256::from(1000000000000000000u64), U256::from(300000), 0.01);
                if is_profitable(loan_amount, U256::from(300000)) {
                    if let Err(e) = execute_flashloan(web3, loan_amount, lending_pool).await {
                        error!("Failed to execute flashloan: {:?}", e);
                    }
                }
            }
        }

        // Wait before next scan
        sleep(Duration::from_secs(check_interval)).await;
    }
}

// Execute the flashloan
pub async fn execute_flashloan(
    web3: &web3::Web3<Http>,
    loaned_amount: U256,
    lending_pool: Address,
) -> Result<(), FlashloanError> {
    let config = load_flashloan_config();
    let asset: Address = config["asset_address"].as_str().unwrap().parse().expect("Invalid address");

    let flashloan_contract = Contract::from_json(
        web3.eth(),
        lending_pool,
        include_bytes!("../abi/aave_flashloan_abi.json")
    ).expect("Invalid Aave flashloan ABI");

    let loan_params = (
        vec![asset],
        vec![loaned_amount],
        vec![0u8],  // Changed from u64 to u8
        asset,
        Vec::<u8>::new(),  // Changed from vec![].as_slice() to Vec::<u8>::new()
        0u16
    );

    let result = flashloan_contract
        .call("flashLoan", loan_params, Address::from_str("YOUR_ADDRESS").unwrap(), Options::default())
        .await;

    match result {
        Ok(_) => {
            info!("Flashloan executed successfully for amount: {:?}", loaned_amount);
            Ok(())
        }
        Err(e) => {
            error!("Flashloan execution failed: {:?}", e);
            Err(FlashloanError::ExecutionFailed(e))
        }
    }
}// Retry logic for flashloan execution
pub async fn execute_flashloan_with_retry(
    web3: &web3::Web3<Http>,
    loaned_amount: U256,
    lending_pool: Address,
    max_retries: u8
) -> Result<(), FlashloanError> {
    let mut attempts = 0;
    let mut delay = 1;

    while attempts < max_retries {
        let result = execute_flashloan(web3, loaned_amount, lending_pool).await;
        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Flashloan execution failed: {}, attempt {}/{}", e, attempts + 1, max_retries);
                attempts += 1;
                sleep(Duration::from_secs(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    Err(FlashloanError::RetriesExceeded)
}

// Get liquidity data from the lending pool
pub async fn get_liquidity_data(
    web3: &web3::Web3<Http>,
    lending_pool: Address
) -> Result<U256, FlashloanError> {
    let flashloan_contract = Contract::from_json(
        web3.eth(),
        lending_pool,
        include_bytes!("../abi/aave_flashloan_abi.json")
    ).expect("Invalid Aave flashloan ABI");

    let result: U256 = flashloan_contract
        .query("getReserveData", (), None, Options::default(), None)
        .await
        .map_err(FlashloanError::ContractError)?;

    Ok(result)
}

// Custom error type for flashloans
#[derive(Error, Debug)]
pub enum FlashloanError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("Contract error: {0}")]
    ContractError(#[from] web3::contract::Error),
    #[error("Execution failed")]
    ExecutionFailed(web3::contract::Error),
    #[error("Retries exceeded for flashloan")]
    RetriesExceeded,
}


