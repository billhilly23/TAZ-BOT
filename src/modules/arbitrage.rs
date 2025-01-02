use serde_json::Value;
use std::fs;
use web3::types::{U256, Address};
use web3::contract::Options;
use web3::contract::Contract;
use log::{error, info};
use thiserror::Error;
use tokio::time::{sleep, Duration};
use web3::transports::Http;
use std::str::FromStr;
use tokio::task::spawn;
use futures::future::join_all;

// Load arbitrage config
fn load_arbitrage_config() -> Value {
    let config_path = "config/arbitrage_config.json";
    let config_data = std::fs::read_to_string(config_path).expect("Unable to read arbitrage config file");
    serde_json::from_str(&config_data).expect("Unable to parse arbitrage config file")
}

// Convert string to Address
fn str_to_address(address: &str) -> Address {
    Address::from_str(address).unwrap()
}

// Dynamic loan calculation for arbitrage opportunities
pub fn calculate_dynamic_loan_amount(expected_profit: U256, gas_fee: U256, slippage: f64) -> U256 {
    let slippage_factor = 1.0 - slippage;
    let max_loan_amount = (expected_profit.low_u64() as f64 * slippage_factor) as u64;
    U256::from(max_loan_amount).saturating_sub(gas_fee)
}

// Profitability Tracking
pub fn is_profitable(profit: U256, gas_fees: U256) -> bool {
    profit > gas_fees
}

// Scan DEX prices and identify arbitrage opportunities
pub async fn scan_for_opportunities(
    web3: web3::Web3<Http>,
    token_pairs: Vec<(Address, Address)>,
    check_interval: u64
) {
    loop {
        let mut tasks = vec![];

        for (token_in, token_out) in token_pairs.iter().cloned() {
            let web3_clone = web3.clone();
            tasks.push(spawn(async move {
                if let Err(e) = check_arbitrage_opportunity(&web3_clone, token_in, token_out).await {
                    error!("Error checking arbitrage opportunity: {:?}", e);
                }
            }));
        }

        join_all(tasks).await;
        sleep(Duration::from_secs(check_interval)).await;
    }
}

// Check arbitrage opportunity between two tokens
pub async fn check_arbitrage_opportunity(
    web3: &web3::Web3<Http>,
    token_in: Address,
    token_out: Address,
) -> Result<(), ArbitrageError> {
    let config = load_arbitrage_config();
    
    let uniswap_router_contract = Contract::from_json(
        web3.eth(),
        str_to_address(&config["uniswap_router_address"].as_str().unwrap()),
        include_bytes!("../abi/uniswap_router_abi.json")
    ).expect("Invalid Uniswap router ABI");

    let sushiswap_router_contract = Contract::from_json(
        web3.eth(),
        str_to_address(&config["sushiswap_router_address"].as_str().unwrap()),
        include_bytes!("../abi/sushiswap_router_abi.json")
    ).expect("Invalid Sushiswap router ABI");

    let price_uniswap = get_token_price(web3, &uniswap_router_contract, token_in, token_out).await?;
    let price_sushiswap = get_token_price(web3, &sushiswap_router_contract, token_in, token_out).await?;

    if price_uniswap > price_sushiswap {
        let profit = price_uniswap - price_sushiswap;
        let gas_fees = U256::from(300000); // Example gas fees
        if is_profitable(profit, gas_fees) {
            info!("Profitable arbitrage opportunity found: Profit: {:?}, Gas: {:?}", profit, gas_fees);
            execute_multi_leg_arbitrage(web3, profit).await?;
        }
    }

    Ok(())
}

// Multi-leg arbitrage logic (A -> B -> C -> A)
pub async fn execute_multi_leg_arbitrage(
    web3: &web3::Web3<Http>,
    loaned_amount: U256
) -> Result<(), ArbitrageError> {
    // Implementation of execute_multi_leg_arbitrage function
    unimplemented!("execute_multi_leg_arbitrage function not implemented")
}

async fn get_token_price(
    web3: &web3::Web3<Http>,
    router_contract: &Contract<Http>,
    token_in: Address,
    token_out: Address,
) -> Result<U256, ArbitrageError> {
    // Implementation of get_token_price function
    // This is a placeholder and should be replaced with actual implementation
    unimplemented!("get_token_price function not implemented")
}    let config = load_arbitrage_config();
    let token_a: Address = config["arbitrage_token_a"].as_str().unwrap().parse().expect("Invalid address");
    let token_b: Address = config["arbitrage_token_b"].as_str().unwrap().parse().expect("Invalid address");
    let token_c: Address = config["arbitrage_token_c"].as_str().unwrap().parse().expect("Invalid address");

    let uniswap_router_contract = Contract::from_json(
        web3.eth(),
        str_to_address(&config["uniswap_router_address"].as_str().unwrap()),
        include_bytes!("../abi/uniswap_router_abi.json")
    ).expect("Invalid Uniswap router ABI");
    let sushiswap_router_contract = Contract::from_json(
        web3.eth(),
        str_to_address(&config["sushiswap_router_address"].as_str().unwrap()),
        include_bytes!("../abi/sushiswap_router_abi.json")
    ).expect("Invalid Sushiswap router ABI");
    pub fn estimate_gas_fees() -> U256 {
        U256::from(300000) // Example gas fees for arbitrage trades
    }
    // Multi-leg arbitrage (A -> B -> C -> A)
    let leg_1_profit = perform_trade(web3, &uniswap_router_contract, token_a, token_b, loaned_amount).await?;
    if is_profitable(leg_1_profit, gas_fees) {
        let leg_2_profit = perform_trade(web3, &sushiswap_router_contract, token_b, token_c, leg_1_profit).await?;
        if is_profitable(leg_2_profit, gas_fees) {
            let final_profit = perform_trade(web3, &uniswap_router_contract, token_c, token_a, leg_2_profit).await?;
            if is_profitable(final_profit, gas_fees) {
                info!("Arbitrage completed successfully with a profit.");
            } else {
                error!("Final leg of arbitrage was not profitable.");
            }
        } else {
            error!("Second leg of arbitrage was not profitable.");
            return Ok(());
        }
    } else {
        error!("First leg of arbitrage was not profitable.");
        return Ok(());
    }        error!("First leg of arbitrage was not profitable.");
    


    Ok(())

// Execute individual trades
pub async fn perform_trade(
    web3: &web3::Web3<Http>,
    router_contract: &Contract<Http>,
    token_in: Address,
    token_out: Address,
    amount_in: U256
) -> Result<U256, ArbitrageError> {
    let gas_fees: U256 = U256::from(300000); // Example gas fees
    let trade_params = (vec![token_in, token_out], amount_in, 1u64);

    let result = router_contract
        .call("swapExactTokensForTokens", trade_params, "YOUR_ADDRESS".parse().unwrap(), Options::default())
        .await;

    match result {
        Ok(output_amount) => {
            info!("Trade executed: {:?}", output_amount);
            Ok(U256::from(output_amount))
        }
        Err(e) => {
            error!("Failed to execute trade: {:?}", e);
            Err(ArbitrageError::ContractError(e))
        }
    }
}

// Retry logic for arbitrage trades
pub async fn execute_arbitrage_with_retry(
    web3: &web3::Web3<Http>,
    loaned_amount: U256,
    max_retries: u8
) -> Result<(), ArbitrageError> {
    let mut attempts = 0;
    let mut delay = 1;

    while attempts < max_retries {
        let result = execute_multi_leg_arbitrage(web3, loaned_amount).await;
        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Arbitrage failed: {}, attempt {}/{}", e, attempts + 1, max_retries);
                attempts += 1;
                sleep(Duration::from_secs(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    Err(ArbitrageError::RetriesExceeded)
}

// Custom error type for arbitrage
#[derive(Error, Debug)]
pub enum ArbitrageError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("Contract error: {0}")]
    ContractError(#[from] web3::contract::Error),
    #[error("Retries exceeded for arbitrage")]
    RetriesExceeded,
}

// Implement conversion for ArbitrageError to Web3 error
impl From<ArbitrageError> for web3::Error {
    fn from(error: ArbitrageError) -> Self {
        web3::Error::Decoder(format!("{:?}", error))
    }
}

