use warp::Filter;
use serde_json::Value;
use std::fs;
use tokio::time::{sleep, Duration};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::task;
use warp::ws::{Message, WebSocket};
use warp::hyper::StatusCode;
use futures_util::{StreamExt, SinkExt};

// Structure to hold the configuration, current status, and profit tracking
struct DashboardState {
    config: Value,
    refresh_interval: u64,
    status: Arc<Mutex<String>>,
    profit: Arc<Mutex<f64>>,  // Added for profit tracking
}

// Load dashboard configuration from file
fn load_dashboard_config() -> Value {
    let config_path = "config/dashboard_config.json";
    let config_data = fs::read_to_string(config_path)
        .expect("Unable to read dashboard config file");
    serde_json::from_str(&config_data).expect("Unable to parse dashboard config file")
}

// Serve static files (HTML, CSS, JS)
async fn serve_static_file(file_path: &str) -> Result<impl warp::Reply, Infallible> {
    let content = fs::read_to_string(file_path).unwrap();
    Ok(warp::reply::html(content))
}

// POST handler to trigger bot strategies
async fn run_strategy(strategy: &str, state: Arc<Mutex<String>>, profit: Arc<Mutex<f64>>) -> Result<impl warp::Reply, Infallible> {
    let mut status = state.lock().unwrap();
    *status = format!("Running {} strategy", strategy);

    // Simulate running the strategy (replace with real logic)
    sleep(Duration::from_secs(3)).await;
    
    // Simulate profit calculation for demo (replace with actual logic)
    let mut profit_value = profit.lock().unwrap();
    *profit_value += 100.0;

    *status = format!("{} strategy completed", strategy);
    Ok(warp::reply::json(&format!("{} strategy executed successfully. Current profit: {}", strategy, *profit_value)))
}

// POST handler to trigger multiple strategies
async fn run_multiple_strategies(state: Arc<Mutex<String>>, profit: Arc<Mutex<f64>>) -> Result<impl warp::Reply, Infallible> {
    let mut status = state.lock().unwrap();
    *status = String::from("Running multiple strategies");

    // Simulate running two strategies in parallel (arbitrage and flashloan)
    let arbitrage_task = task::spawn(run_strategy("arbitrage", state.clone(), profit.clone()));
    let flashloan_task = task::spawn(run_strategy("flashloan", state.clone(), profit.clone()));

    // Wait for both tasks to complete
    let _ = tokio::join!(arbitrage_task, flashloan_task);

    *status = String::from("Multiple strategies completed");
    Ok(warp::reply::json(&format!("Multiple strategies executed successfully. Current profit: {}", *profit.lock().unwrap())))
}

// Real-time WebSocket monitoring for updates (e.g., flashloan status, profit)
async fn handle_websocket(ws: WebSocket, state: Arc<Mutex<String>>, profit: Arc<Mutex<f64>>) {
    let (mut tx, mut rx) = ws.split();

    while let Some(result) = rx.next().await {
        if result.is_ok() {
            let status = state.lock().unwrap().clone();
            let profit_value = *profit.lock().unwrap();
            let message = format!("Status: {}, Profit: {}", status, profit_value);

            if tx.send(Message::text(message)).await.is_err() {
                break;
            }
        }
    }
}

// HTML dashboard handler
async fn dashboard_handler() -> Result<impl warp::Reply, Infallible> {
    serve_static_file("static/dashboard.html").await
}

// Run the Warp server and handle routes
#[tokio::main]
async fn main() {
    let config = load_dashboard_config();
    let refresh_interval = config["refresh_interval"].as_u64().unwrap_or(60);
    let state = Arc::new(Mutex::new(String::from("Ready")));
    let profit = Arc::new(Mutex::new(0.0)); // Initialize profit tracking

    let state_filter = warp::any().map(move || state.clone());
    let profit_filter = warp::any().map(move || profit.clone());

    // WebSocket route
    let websocket_route = warp::path("ws")
        .and(warp::ws())
        .and(state_filter.clone())
        .and(profit_filter.clone())
        .map(|ws: warp::ws::Ws, state, profit| {
            ws.on_upgrade(move |socket| handle_websocket(socket, state, profit))
        });

    // Route to serve the dashboard HTML
    let dashboard = warp::path("dashboard")
        .and(warp::get())
        .and_then(dashboard_handler);

    // Serve static files (CSS, JS)
    let css = warp::path("dashboard.css")
        .and(warp::get())
        .and_then(|| serve_static_file("static/dashboard.css"));

    let js = warp::path("dashboard.js")
        .and(warp::get())
        .and_then(|| serve_static_file("static/dashboard.js"));

    // Route to handle POST requests for bot strategies
    let run_arbitrage = warp::path("run-arbitrage")
        .and(warp::post())
        .and(state_filter.clone())
        .and(profit_filter.clone())
        .and_then(run_strategy);

    let run_flashloan = warp::path("run-flashloan")
        .and(warp::post())
        .and(state_filter.clone())
        .and(profit_filter.clone())
        .and_then(run_strategy);

    let run_multiple = warp::path("run-multiple")
        .and(warp::post())
        .and(state_filter.clone())
        .and(profit_filter.clone())
        .and_then(run_multiple_strategies);

    // Run Warp server
    let routes = websocket_route
        .or(dashboard)
        .or(css)
        .or(js)
        .or(run_arbitrage)
        .or(run_flashloan)
        .or(run_multiple);

    warp::serve(routes)
        .run(([127, 0, 0, 1], config["port"].as_u64().unwrap_or(8080) as u16))
        .await;
}


