use web3::types::{H160, U256};
use web3::contract::{Contract, Options};
use web3::transports::Http;
use serde_json::Value;
use thiserror::Error;
use std::fs;
use tokio::time::{sleep, Duration};
use log::{info, error};

// Chainlink AggregatorV3Interface ABI (to fetch price from Chainlink price feed)
const CHAINLINK_AGGREGATOR_ABI: &[u8] = include_bytes!("abi/chainlink_aggregator_abi.json");

// Load configuration for liquidation
fn load_liquidation_config() -> Value {
    let config_path = "config/liquidation_config.json";
    let config_data = fs::read_to_string(config_path).expect("Unable to read liquidation config file");
    serde_json::from_str(&config_data).expect("Unable to parse liquidation config file")
}

// Custom error type for liquidation
#[derive(Error, Debug)]
pub enum LiquidationError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("Contract error: {0}")]
    ContractError(#[from] web3::contract::Error),
    #[error("ABI error: {0}")]
    ABIError(#[from] web3::ethabi::Error),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Retries exceeded for liquidation execution")]
    RetriesExceeded,
}

// Implement conversion for LiquidationError to Web3 error
impl From<LiquidationError> for web3::Error {
    fn from(error: LiquidationError) -> Self {
        web3::Error::Decoder(format!("{:?}", error))
    }
}

// Liquidation struct to hold both Aave and Compound settings
struct Liquidation<'a> {
    aave_pool: Contract<&'a Http>,
    compound_comptroller: Contract<&'a Http>,
    ctoken_collateral: Contract<&'a Http>,
}

impl<'a> Liquidation<'a> {
    // Initialize Liquidation struct with Aave and Compound contracts
    pub fn new(web3: &'a web3::Web3<Http>, config: &Value) -> Result<Self, LiquidationError> {
        let aave_pool_address: H160 = config["aave_pool"].as_str().unwrap().parse().expect("Invalid address");
        let compound_comptroller_address: H160 = config["compound_comptroller"].as_str().unwrap().parse().expect("Invalid address");
        let ctoken_collateral_address: H160 = config["ctoken_collateral"].as_str().unwrap().parse().expect("Invalid address");

        let aave_pool = Contract::from_json(web3.eth(), aave_pool_address, include_bytes!("abi/aave_pool_abi.json"))?;
        let compound_comptroller = Contract::from_json(web3.eth(), compound_comptroller_address, include_bytes!("abi/compound_comptroller_abi.json"))?;
        let ctoken_collateral = Contract::from_json(web3.eth(), ctoken_collateral_address, include_bytes!("abi/ctoken_abi.json"))?;

        Ok(Liquidation { aave_pool, compound_comptroller, ctoken_collateral })
    }

    // Track debt ratios across multiple protocols and check if the account is near liquidation
    pub async fn track_debt_ratios(&self, borrower_address: H160) -> Result<bool, LiquidationError> {
        // Fetch health factor from Aave
        let health_factor: U256 = self.aave_pool
            .query("getHealthFactor", borrower_address, None, Options::default(), None)
            .await
            .map_err(LiquidationError::ContractError)?;

        // Fetch the liquidity ratio from Compound (as an example, you would need the specific Compound method)
        let liquidity_ratio: U256 = self.compound_comptroller
            .query("getAccountLiquidity", borrower_address, None, Options::default(), None)
            .await
            .map_err(LiquidationError::ContractError)?;

        let is_near_liquidation = health_factor < U256::from(1_000_000_000_000_000_000u128) || liquidity_ratio.is_zero();

        Ok(is_near_liquidation)
    }

    // Function to calculate profit from liquidating a borrower
    pub async fn calculate_liquidation_profit(
        &self,
        collateral_asset: H160,
        debt_covered: U256,
        price_feed_address: H160
    ) -> Result<U256, LiquidationError> {
        let collateral_price: U256 = self.get_asset_price(price_feed_address).await?;
        let seized_collateral_value = collateral_price * debt_covered;
        let profit = seized_collateral_value.saturating_sub(debt_covered);

        Ok(profit)
    }

    // Retry logic for liquidation in case of failure
    pub async fn execute_liquidation_with_retry(
        &self,
        borrower_address: H160,
        debt_covered: U256,
        collateral_asset: H160,
        max_retries: u8
    ) -> Result<(), LiquidationError> {
        let mut attempts = 0;
        let mut delay = 1;

        while attempts < max_retries {
            let result = self.execute_liquidation(borrower_address, debt_covered, collateral_asset).await;
            match result {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!("Liquidation failed: {}, attempt {}/{}", e, attempts + 1, max_retries);
                    attempts += 1;
                    sleep(Duration::from_secs(delay)).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }

        Err(LiquidationError::RetriesExceeded)
    }

    // Execute liquidation by interacting with the Aave and Compound contracts
    pub async fn execute_liquidation(
        &self,
        borrower_address: H160,
        debt_covered: U256,
        collateral_asset: H160
    ) -> Result<(), LiquidationError> {
        let flashloan_result = self.request_flashloan(debt_covered).await?;
        if flashloan_result.is_ok() {
            info!("Executing liquidation for borrower: {:?}", borrower_address);
            Ok(())
        } else {
            error!("Failed to request flashloan for liquidation");
            Err(LiquidationError::ContractError(flashloan_result.unwrap_err()))
        }
    }

    // Request flashloan function, integrated from flashloan module
    async fn request_flashloan(&self, amount: U256) -> Result<(), LiquidationError> {
        info!("Requesting flashloan for amount: {:?}", amount);
        // Integrate live flashloan contract interaction here
        Ok(())
    }

    // Function to map an asset to its Chainlink price feed address
    pub fn get_chainlink_price_feed_address(&self, asset: H160) -> Result<H160, LiquidationError> {
        let price_feed_address: H160 = match asset {
            eth_address if eth_address == "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".parse().unwrap() => {
                "0x5f4ec3df9cbd43714fe2740f5e3616155c5b8419".parse().unwrap()
            }
            dai_address if dai_address == "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse().unwrap() => {
                "0xAed0c38402a5d19df6E4c03F4E2DceD6e29c1ee9".parse().unwrap()
            }
            usdc_address if usdc_address == "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606EB48".parse().unwrap() => {
                "0x8fFfFfd4AfB6115b954Bd326cbe7B4BA576818f6".parse().unwrap()
            }
            _ => return Err(LiquidationError::Web3Error(web3::Error::Decoder("Unsupported asset".into()))),
        };

        Ok(price_feed_address)
    }

    // Function to get asset price from Chainlink price feed
    async fn get_asset_price(&self, price_feed_address: H160) -> Result<U256, LiquidationError> {
        let chainlink_contract = Contract::from_json(self.aave_pool.web3().eth(), price_feed_address, CHAINLINK_AGGREGATOR_ABI)?;
        let price: U256 = chainlink_contract.query("latestAnswer", (), None, Options::default(), None).await?;
        Ok(price)
    }
}


