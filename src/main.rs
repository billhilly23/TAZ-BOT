use serde_json::Value;
use std::fs;
use web3::transports::Http;
use web3::types::U256;
use web3::Web3;
use tokio::task;
use log::{info, error};

// Import modules for different strategies
mod modules {
    pub mod arbitrage;
    pub mod flashloan;
    pub mod frontrunning;
    pub mod liquidation;
    pub mod sandwich;
    pub mod hft;
}

// Load global config file
fn load_global_config() -> Value {
    let config_path = "config/global_config.json";
    let config_data = fs::read_to_string(config_path)
        .expect("Unable to read global config file");
    serde_json::from_str(&config_data).expect("Unable to parse global config file")
}

// Load individual strategy config based on global config
fn load_strategy_config(strategy_name: &str) -> Value {
    let global_config = load_global_config();
    let strategy_path = global_config["strategies"][strategy_name]["config_path"]
        .as_str()
        .expect("Strategy config path not found");
    let config_data = fs::read_to_string(strategy_path)
        .expect("Unable to read strategy config file");
    serde_json::from_str(&config_data).expect("Unable to parse strategy config file")
}

#[tokio::main]
async fn main() -> web3::Result<()> {
    // Load global configuration
    let global_config = load_global_config();
    let infura_project_id = global_config["infura_project_id"].as_str().unwrap();
    let network = global_config["network"].as_str().unwrap();
    let eth_node_url = format!("https://{}.infura.io/v3/{}", network, infura_project_id);

    let transport = Http::new(ð_node_url)?;
    let web3 = Web3::new(transport);
    let web3 = std::sync::Arc::new(web3);

    let default_gas_limit = global_config["default_gas_limit"].as_u64().unwrap_or(5000000);
    let bot_mode = global_config["bot_mode"].as_str().unwrap();

    // Monitoring (if enabled)
    if global_config["monitoring_enabled"].as_bool().unwrap_or(false) {
        info!("Monitoring enabled");
        task::spawn(async {
            // Add monitoring logic if necessary
        });
    }

    // Run strategies based on bot mode
    match bot_mode {
        "arbitrage" => {
            info!("Running Arbitrage Strategy");
            let arbitrage_config = load_strategy_config("arbitrage");
            modules::arbitrage::execute_arbitrage_with_retry(web3.clone(), U256::zero(), 3).await.unwrap();
        }
        "flashloan" => {
            info!("Running Flashloan Strategy");
            let flashloan_config = load_strategy_config("flashloan");
            let asset_address = flashloan_config["asset_address"].as_str().unwrap().parse().unwrap();
            modules::flashloan::execute_flashloan(web3.clone(), U256::zero(), asset_address).await.unwrap();
        }
        "frontrunning" => {
            info!("Running Frontrunning Strategy");
            let frontrunning_config = load_strategy_config("frontrunning");
            let transactions = modules::frontrunning::fetch_mempool_transactions(web3.clone()).await;
            // Process the fetched transactions as needed
        }
        "liquidation" => {
            info!("Running Liquidation Strategy");
            let liquidation_config = load_strategy_config("liquidation");
            let borrower_address = liquidation_config["borrower_address"].as_str().unwrap().parse().unwrap();
            let collateral_asset = liquidation_config["collateral_asset"].as_str().unwrap().parse().unwrap();
            modules::liquidation::execute_liquidation(web3.clone(), borrower_address, U256::zero(), collateral_asset).await.unwrap();
        }
        "sandwich" => {
            info!("Running Sandwich Attack Strategy");
            let sandwich_config = load_strategy_config("sandwich");
            modules::sandwich::execute_sandwich_attack_with_retry(web3.clone(), U256::zero(), 3).await.unwrap();
        }
        "hft" => {
            info!("Running HFT Strategy");
            let hft_config = load_strategy_config("hft");
            modules::hft::execute_hft(web3.clone()).await.unwrap();
        }
        "multi" | "all" => {
            info!("Running All Enabled Strategies");
            let enabled_strategies = global_config["strategies"]
                .as_object()
                .unwrap()
                .iter()
                .filter(|(_, strategy)| strategy["enabled"].as_bool().unwrap_or(false))
                .map(|(name, _)| name.to_string())
                .collect::<Vec<_>>();

            for strategy in enabled_strategies {
                match strategy.as_str() {
                    "arbitrage" => {
                        info!("Running Arbitrage");
                        let arbitrage_config = load_strategy_config("arbitrage");
                        modules::arbitrage::execute_arbitrage_with_retry(web3.clone(), U256::zero(), 3).await.unwrap();
                    }
                    "flashloan" => {
                        info!("Running Flashloan");
                        let flashloan_config = load_strategy_config("flashloan");
                        let asset_address = flashloan_config["asset_address"].as_str().unwrap().parse().unwrap();

                        modules::flashloan::execute_flashloan(web3.clone(), U256::zero(), asset_address).await.unwrap();
                    }
                    "frontrunning" => {
                        info!("Running Frontrunning");
                        let frontrunning_config = load_strategy_config("frontrunning");
                        let transactions = modules::frontrunning::fetch_mempool_transactions(web3.clone()).await;
                        // Process the fetched transactions as needed
                    }
                    "liquidation" => {
                        info!("Running Liquidation");
                        let liquidation_config = load_strategy_config("liquidation");
                        let borrower_address = liquidation_config["borrower_address"].as_str().unwrap().parse().unwrap();
                        let collateral_asset = liquidation_config["collateral_asset"].as_str().unwrap().parse().unwrap();
                        modules::liquidation::execute_liquidation(web3.clone(), borrower_address, U256::zero(), collateral_asset).await.unwrap();
                    }
                    "sandwich" => {
                        info!("Running Sandwich Attack");
                        let sandwich_config = load_strategy_config("sandwich");
                        modules::sandwich::execute_sandwich_attack_with_retry(web3.clone(), U256::zero(), 3).await.unwrap();
                    }
                    "hft" => {
                        info!("Running HFT");
                        let hft_config = load_strategy_config("hft");
                        modules::hft::execute_hft(web3.clone()).await.unwrap();
                    }
                    _ => error!("Unknown strategy: {}", strategy),
                }
            }
        }
        _ => {
            error!("Invalid bot mode: {}", bot_mode);
        }
    }

    Ok(())
}