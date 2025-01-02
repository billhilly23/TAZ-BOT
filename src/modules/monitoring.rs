use serde_json::Value;
use std::fs;
use web3::types::{U256, Address, H160};
use web3::contract::Options;
use web3::contract::Contract;
use web3::transports::Http;
use log::{error, info};
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use thiserror::Error;
use tokio::time::{sleep, Duration};
use std::str::FromStr;
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};

// Load the monitoring configuration
fn load_monitoring_config() -> Value {
    let config_path = "config/monitoring_config.json";
    let config_data = fs::read_to_string(config_path)
        .expect("Unable to read monitoring config file");
    serde_json::from_str(&config_data).expect("Unable to parse monitoring config file")
}

// WebSocket-based monitoring for real-time events (e.g., pending transactions)
pub async fn monitor_websocket_for_events() -> Result<(), MonitoringError> {
    let config = load_monitoring_config();
    let websocket_url = config["websocket_url"].as_str().expect("WebSocket URL not found");

    let (ws_stream, _) = connect_async(websocket_url).await.expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    write.send("subscribe_to_events".into()).await.expect("Failed to send WebSocket message");

    while let Some(msg) = read.next().await {
        let msg_text = msg.expect("Error reading WebSocket message").to_text().unwrap();
        info!("Received WebSocket message: {}", msg_text);
        // Implement logic for handling real-time events
    }

    Ok(())
}

// Send email notification with retry logic
pub fn send_email_notification(subject: &str, body: &str) -> Result<(), MonitoringError> {
    let config = load_monitoring_config();
    let smtp_user = config["smtp_user"].as_str().expect("SMTP user not found");
    let smtp_pass = config["smtp_pass"].as_str().expect("SMTP pass not found");
    let recipient = config["recipient_email"].as_str().expect("Recipient email not found");

    let email = Message::builder()
        .from("Monitoring System <monitoring@example.com>".parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .expect("Unable to create email");

    let creds = Credentials::new(smtp_user.to_string(), smtp_pass.to_string());

    let mailer = SmtpTransport::relay("smtp.example.com")
        .unwrap()
        .credentials(creds)
        .build();

    for _ in 0..3 {  // Retry logic
        if mailer.send(&email).is_ok() {
            info!("Email sent successfully.");
            return Ok(());
        }
        error!("Failed to send email. Retrying...");
        sleep(Duration::from_secs(5)).await;
    }

    Err(MonitoringError::EmailError(lettre::error::Error::Client))
}

// Send SMS notification with retry logic (Twilio example)
pub fn send_sms_notification(body: &str) -> Result<(), MonitoringError> {
    let config = load_monitoring_config();
    let twilio_sid = config["twilio_sid"].as_str().expect("Twilio SID not found");
    let twilio_token = config["twilio_token"].as_str().expect("Twilio token not found");
    let recipient_phone = config["recipient_phone"].as_str().expect("Recipient phone not found");

    for _ in 0..3 {  // Retry logic
        let result = twilio::OutboundMessage::new(twilio_sid, twilio_token)
            .to(recipient_phone)
            .body(body)
            .send();

        if result.is_ok() {
            info!("SMS sent successfully.");
            return Ok(());
        }
        error!("Failed to send SMS. Retrying...");
        sleep(Duration::from_secs(5)).await;
    }

    Err(MonitoringError::TwilioError(twilio::error::Error::Client))
}

// Calculate and monitor real-time profit for each module
pub async fn monitor_real_time_profit(web3: &web3::Web3<Http>, modules: Vec<H160>) -> f64 {
    let mut total_profit: f64 = 0.0;
    let config = load_monitoring_config();

    for module in modules {
        let initial_balance = web3.eth().balance(module, None).await.expect("Failed to fetch initial balance");
        let current_balance = web3.eth().balance(module, None).await.expect("Failed to fetch current balance");

        let initial_balance_f64 = initial_balance.low_u64() as f64 / 1e18;
        let current_balance_f64 = current_balance.low_u64() as f64 / 1e18;

        let profit = current_balance_f64 - initial_balance_f64;
        total_profit += profit;
    }

    info!("Real-time profit: {}", total_profit);
    total_profit
}

// Fetch the real-time gas usage based on recent transactions sent by the bot
pub async fn get_real_time_gas_usage(web3: &web3::Web3<Http>, bot_address: H160) -> f64 {
    let mut total_gas_used: f64 = 0.0;

    // Example: query the last 100 transactions from the bot
    let tx_count = web3
        .eth()
        .transaction_count(bot_address, None)
        .await
        .expect("Failed to get transaction count");

    let start_tx_count = tx_count.saturating_sub(U256::from(100));

    for nonce in start_tx_count.low_u64()..tx_count.low_u64() {
        let tx_hash = web3.eth().transaction_by_hash(H160::from_low_u64_be(nonce)).await;
        if let Ok(Some(receipt)) = web3.eth().transaction_receipt(tx_hash).await {
            let gas_price: U256 = receipt.gas_used.unwrap_or(U256::zero());
            total_gas_used += gas_price.low_u64() as f64;
        }
    }

    info!("Real-time gas usage: {}", total_gas_used);
    total_gas_used
}

// Custom error type for monitoring
#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("Email error: {0}")]
    EmailError(#[from] lettre::error::Error),
    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("Twilio error: {0}")]
    TwilioError(#[from] twilio::error::Error),
}

// Implement conversion for MonitoringError to Web3 error
impl From<MonitoringError> for web3::Error {
    fn from(error: MonitoringError) -> Self {
        web3::Error::Decoder(format!("{:?}", error))
    }
}

