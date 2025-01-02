use serde_json::Value;
use std::fs;
use web3::types::{U256, Address, TransactionRequest, H160};
use web3::contract::{Contract, Options};
use log::{error, info};
use tokio::task;
use thiserror::Error;
use tokio::time::{sleep, Duration};
use chrono::Utc;
use web3::transports::WebSocket;
use web3::futures::StreamExt;

// Load the sandwich configuration
fn load_sandwich_config() -> Value {
    let config_path = "config/sandwich_config.json";
    let config_data = fs::read_to_string(config_path)
        .expect("Unable to read sandwich config file");
    let config: Value = serde_json::from_str(&config_data)
        .expect("Unable to parse sandwich config file");
    config
}

// Dynamic flash loan calculation for sandwich attacks
pub fn calculate_dynamic_loan_amount(amount_in: U256, gas_fee: U256, slippage: f64, min_profit: U256) -> U256 {
    let slippage_factor = 1.0 - slippage;
    let estimated_profit = amount_in.low_u64() as f64 * slippage_factor;
    let max_loan_amount = (estimated_profit - gas_fee.low_u64() as f64) as u64;
    U256::from(max_loan_amount).max(min_profit)
}

// Real-time monitoring of the mempool for large trades
pub async fn monitor_mempool_for_large_transactions(
    websocket_url: &str,
    min_tx_value: U256
) -> Result<H160, SandwichError> {
    info!("Monitoring mempool for large transactions...");

    // Initialize a WebSocket connection to listen to pending transactions
    let websocket = WebSocket::new(websocket_url).await?;
    let web3 = web3::Web3::new(websocket);

    // Subscribe to pending transactions
    let mut pending_tx_stream = web3.eth_subscribe().subscribe_new_pending_transactions().await?;

    // Loop over the pending transactions
    while let Some(pending_tx) = pending_tx_stream.next().await {
        match pending_tx {
            Ok(tx_hash) => {
                // Fetch the transaction details
                if let Ok(tx) = web3.eth().transaction(TransactionRequest::new().hash(tx_hash)).await {
                    if let Some(transaction) = tx {
                        // Check the transaction value
                        if transaction.value >= min_tx_value {
                            info!(
                                "Detected large transaction: {:?}, Value: {:?}",
                                transaction.from, transaction.value
                            );
                            return Ok(transaction.from); // Return the sender address of the large transaction
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error receiving pending transaction: {:?}", e);
                sleep(Duration::from_secs(1)).await; // Small delay before retrying
            }
        }
    }

    error!("No large transactions detected in mempool");
    Err(SandwichError::NoLargeTrades)
}

// Check if the sandwich attack will be profitable before execution
pub fn is_profitable(flashloan_amount: U256, gas_fee: U256, expected_profit: U256) -> bool {
    expected_profit > (flashloan_amount + gas_fee)
}

// Request a flash loan
pub async fn request_flash_loan(
    web3: web3::Web3<web3::transports::Http>,
    amount: U256
) -> Result<(), SandwichError> {
    let aave_flashloan_contract = Contract::from_json(
        web3.eth(),
        "AAVE_FLASHLOAN_CONTRACT_ADDRESS".parse().unwrap(),
        include_bytes!("abi/aave_flashloan_abi.json"),
    )?;

    let params = (
        vec!["TOKEN_ADDRESS".parse().unwrap()],
        vec![amount],
        vec![0],
        "SENDER_ADDRESS".parse().unwrap(),
        vec![0u8],
    );

    aave_flashloan_contract
        .call("flashLoan", params, Address::zero(), Options::default())
        .await?;

    Ok(())
}

// Execute sandwich attack across multiple DEXs
pub async fn execute_sandwich_attack(
    web3: web3::Web3<web3::transports::Http>,
    flashloan_amount: U256
) -> Result<(), SandwichError> {
    let config = load_sandwich_config();
    let uniswap_router_address: Address = config["uniswap_router_address"].as_str().unwrap().parse().expect("Invalid address");
    let sushiswap_router_address: Address = config["sushiswap_router_address"].as_str().unwrap().parse().expect("Invalid address");

    let uniswap_router_contract = Contract::from_json(
        web3.eth(),
        uniswap_router_address,
        include_bytes!("abi/uniswap_router_abi.json")
    )?;

    let sushiswap_router_contract = Contract::from_json(
        web3.eth(),
        sushiswap_router_address,
        include_bytes!("abi/sushiswap_router_abi.json")
    )?;

    let path = vec!["TOKEN_IN_ADDRESS".parse().unwrap(), "TOKEN_OUT_ADDRESS".parse().unwrap()];
    let recipient = "SENDER_ADDRESS".parse().unwrap();
    let deadline = U256::from(Utc::now().timestamp() + 600);

    // **Front-running transaction**
    let front_run_tx = uniswap_router_contract
        .call("swapExactTokensForTokens", (flashloan_amount, U256::from(1), path.clone(), recipient, deadline), Options::default(), None)
        .await?;

    info!("Front-running transaction executed: {:?}", front_run_tx);

    // **Back-running transaction**
    let back_run_tx = sushiswap_router_contract
        .call("swapExactTokensForTokens", (flashloan_amount, U256::from(1), path, recipient, deadline), Options::default(), None)
        .await?;

    info!("Back-running transaction executed: {:?}", back_run_tx);

    Ok(())
}

// Retry logic for sandwich attacks
pub async fn execute_sandwich_attack_with_retry(
    web3: web3::Web3<web3::transports::Http>,
    flashloan_amount: U256,
    max_retries: u8
) -> Result<(), SandwichError> {
    let mut attempts = 0;
    let mut delay = 1;

    while attempts < max_retries {
        let result = execute_sandwich_attack(web3.clone(), flashloan_amount).await;
        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Sandwich attack failed: {}, attempt {}/{}", e, attempts + 1, max_retries);
                attempts += 1;
                sleep(Duration::from_secs(delay)).await;
                delay *= 2;
            }
        }
    }

    Err(SandwichError::RetriesExceeded)
}

// Repay flash loan
pub async fn repay_flash_loan(
    web3: web3::Web3<web3::transports::Http>,
    flashloan_amount: U256
) -> Result<(), SandwichError> {
    let aave_flashloan_contract = Contract::from_json(
        web3.eth(),
        "AAVE_FLASHLOAN_CONTRACT_ADDRESS".parse().unwrap(),
        include_bytes!("abi/aave_flashloan_abi.json"),
    )?;

    let repay_amount = flashloan_amount + (flashloan_amount / U256::from(1000)); 

    aave_flashloan_contract
        .call("repay", ("TOKEN_ADDRESS".parse().unwrap(), repay_amount, "SENDER_ADDRESS".parse().unwrap()), Options::default(), None)
        .await?;

    info!("Flash loan repaid: {:?}", repay_amount);
    Ok(())
}

// Define errors for the sandwich attack process
#[derive(Error, Debug)]
pub enum SandwichError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("Contract error: {0}")]
    ContractError(#[from] web3::contract::Error),
    #[error("ABI error: {0}")]
    ABIError(#[from] web3::ethabi::Error),
    #[error("Retries exceeded for sandwich attack")]
    RetriesExceeded,
    #[error("No large trades detected")]
    NoLargeTrades,
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

// Convert SandwichError to Web3 error
impl From<SandwichError> for web3::Error {
    fn from(error: SandwichError) -> Self {
        web3::Error::Decoder(error.to_string())
    }
}


