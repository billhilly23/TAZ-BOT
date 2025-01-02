use serde_json::Value;
use std::fs;
use web3::types::{U256, Transaction, Address};
use web3::contract::Contract;
use log::{error, info};
use thiserror::Error;
use tokio::time::{sleep, Duration};
use web3::transports::Http;
use web3::contract::Options;
use std::str::FromStr;
use web3::ethabi::ethereum_types::H256;

// Load frontrunning config
fn load_frontrunning_config() -> Value {
    let config_path = "config/frontrunning_config.json";
    let config_data = std::fs::read_to_string(config_path).expect("Unable to read frontrunning config file");
    serde_json::from_str(&config_data).expect("Unable to parse frontrunning config file")
}

// Convert string to Address
fn str_to_address(address: &str) -> Address {
    Address::from_str(address).unwrap()
}

// Monitor the mempool for large transactions
pub async fn monitor_mempool(
    web3: &web3::Web3<Http>,
    threshold_amount: U256,
    gas_fee_limit: U256,
    check_interval: u64
) {
    loop {
        let pending_transactions = fetch_mempool_transactions(web3).await;

        for transaction in pending_transactions {
            let tx_value = U256::from(transaction.value);
            
            // Filter transactions above the threshold
            if tx_value > threshold_amount {
                let potential_profit = calculate_potential_profit(tx_value, gas_fee_limit);
                
                if is_profitable(potential_profit, gas_fee_limit) {
                    info!("Profitable frontrunning opportunity detected: {:?}", transaction.hash);
                    if let Err(e) = execute_frontrunning(web3, transaction).await {
                        error!("Frontrunning execution failed: {:?}", e);
                    }
                }
            }
        }

        sleep(Duration::from_secs(check_interval)).await;
    }
}

// Fetch pending transactions from the mempool
pub async fn fetch_mempool_transactions(
    web3: &web3::Web3<Http>
) -> Vec<Transaction> {
    let mut pending_txs = Vec::new();
    if let Ok(block) = web3.eth().block(BlockId::Pending).await {
        if let Some(block) = block {
            for tx_hash in block.transactions {
                if let Ok(Some(tx)) = web3.eth().transaction(tx_hash).await {
                    pending_txs.push(tx);
                }
            }
        }
    }
    pending_txs
}
// Calculate the profit potential for frontrunning a transaction
pub fn calculate_potential_profit(
    transaction_value: U256,
    gas_fee_limit: U256
) -> U256 {
    let slippage_factor = 0.01;  // Example: 1% slippage
    let potential_profit = transaction_value - (transaction_value * U256::from_f64(slippage_factor).unwrap());
    potential_profit.saturating_sub(gas_fee_limit)
}

// Check if the transaction is profitable based on gas fees and slippage
pub fn is_profitable(profit: U256, gas_fees: U256) -> bool {
    profit > gas_fees
}

// Execute the frontrunning transaction
pub async fn execute_frontrunning(
    web3: &web3::Web3<Http>,
    target_transaction: Transaction
) -> Result<(), FrontrunningError> {
    let config = load_frontrunning_config();
    let token_in: Address = config["token_in"].as_str().unwrap().parse().expect("Invalid address");
    let token_out: Address = config["token_out"].as_str().unwrap().parse().expect("Invalid address");

    let uniswap_router_contract = Contract::from_json(
        web3.eth(),
        str_to_address(&config["uniswap_router_address"].as_str().unwrap()),
        include_bytes!("../abi/uniswap_router_abi.json")
    ).expect("Invalid Uniswap router ABI");

    let gas_price = U256::from(20000000000u64); // Example gas price (20 Gwei)

    let tx_hash = target_transaction.hash;
    let trade_params = (vec![token_in, token_out], target_transaction.value, 1u64);

    let result = uniswap_router_contract
        .call("swapExactTokensForTokens", trade_params, "YOUR_ADDRESS".parse().unwrap(), Options::with(|opt| {
            opt.gas_price = Some(gas_price);
        }))
        .await;

    match result {
        Ok(_) => {
            info!("Frontrunning transaction executed successfully: {:?}", tx_hash);
            Ok(())
        }
        Err(e) => {
            error!("Failed to execute frontrunning transaction: {:?}", e);
            Err(FrontrunningError::ContractError(e))
        }
    }
}

// Retry logic for frontrunning trades
pub async fn execute_frontrunning_with_retry(
    web3: &web3::Web3<Http>,
    target_transaction: Transaction,
    max_retries: u8
) -> Result<(), FrontrunningError> {
    let mut attempts = 0;
    let mut delay = 1;

    while attempts < max_retries {
        let result = execute_frontrunning(web3, target_transaction).await;
        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Frontrunning failed: {}, attempt {}/{}", e, attempts + 1, max_retries);
                attempts += 1;
                sleep(Duration::from_secs(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    Err(FrontrunningError::RetriesExceeded)
}

// Custom error type for frontrunning
#[derive(Error, Debug)]
pub enum FrontrunningError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("Contract error: {0}")]
    ContractError(#[from] web3::contract::Error),
    #[error("Retries exceeded for frontrunning")]
    RetriesExceeded,
}


