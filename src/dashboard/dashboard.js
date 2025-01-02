// Store the configuration data in local storage
function saveConfiguration(strategy, config) {
    localStorage.setItem(strategy, JSON.stringify(config));
}

// Retrieve the configuration data from local storage
function getConfiguration(strategy) {
    const config = localStorage.getItem(strategy);
    return config ? JSON.parse(config) : {};
}

// Update the status and profit displays
function updateStatus(message) {
    document.getElementById('status-message').innerText = message;
}

function updateProfit(amount) {
    document.getElementById('profit-amount').innerText = '$' + amount.toFixed(2);
}

// Generic function to send requests to the backend
function runStrategy(strategy, config) {
    fetch(`/run-${strategy}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(config)
    })
    .then(response => response.json())
    .then(data => {
        updateStatus(`${strategy} strategy completed`);
        if (data.profit) {
            updateProfit(data.profit);
        }
    })
    .catch(error => {
        console.error('Error:', error);
        updateStatus(`${strategy} strategy failed`);
    });
}

// Arbitrage strategy
function runArbitrage() {
    const config = {
        contractAddress: document.getElementById('arbitrage-contract').value,
        slippage: parseFloat(document.getElementById('arbitrage-slippage').value),
        gasFee: parseInt(document.getElementById('arbitrage-gas').value),
        tradeSize: parseFloat(document.getElementById('arbitrage-dynamic-size').value)
    };
    saveConfiguration('arbitrage', config);
    runStrategy('arbitrage', config);
}

// Flashloan strategy
function runFlashloan() {
    const config = {
        contractAddress: document.getElementById('flashloan-contract').value,
        loanAmount: parseFloat(document.getElementById('flashloan-amount').value),
        gasFee: parseInt(document.getElementById('flashloan-gas').value),
        dynamicSize: parseFloat(document.getElementById('flashloan-dynamic-size').value)
    };
    saveConfiguration('flashloan', config);
    runStrategy('flashloan', config);
}

// Frontrunning strategy
function runFrontrunning() {
    const config = {
        contractAddress: document.getElementById('frontrunning-contract').value,
        expectedProfit: parseFloat(document.getElementById('frontrunning-profit').value),
        gasFee: parseInt(document.getElementById('frontrunning-gas').value),
        tradeSize: parseFloat(document.getElementById('frontrunning-dynamic-size').value)
    };
    saveConfiguration('frontrunning', config);
    runStrategy('frontrunning', config);
}

// HFT strategy
function runHFT() {
    const config = {
        contractAddress: document.getElementById('hft-contract').value,
        tradeSize: parseFloat(document.getElementById('hft-dynamic-size').value),
        gasFee: parseInt(document.getElementById('hft-gas').value)
    };
    saveConfiguration('hft', config);
    runStrategy('hft', config);
}

// Liquidation strategy
function runLiquidation() {
    const config = {
        contractAddress: document.getElementById('liquidation-contract').value,
        debtRatio: parseFloat(document.getElementById('liquidation-debt-ratio').value),
        gasFee: parseInt(document.getElementById('liquidation-gas').value)
    };
    saveConfiguration('liquidation', config);
    runStrategy('liquidation', config);
}

// Sandwich Attack strategy
function runSandwich() {
    const config = {
        contractAddress: document.getElementById('sandwich-contract').value,
        slippage: parseFloat(document.getElementById('sandwich-slippage').value),
        gasFee: parseInt(document.getElementById('sandwich-gas').value)
    };
    saveConfiguration('sandwich', config);
    runStrategy('sandwich', config);
}

// Run all strategies
function runMultipleStrategies() {
    runArbitrage();
    runFlashloan();
    runFrontrunning();
    runHFT();
    runLiquidation();
    runSandwich();
}

// Load configurations from local storage on page load
window.onload = function() {
    const strategies = ['arbitrage', 'flashloan', 'frontrunning', 'hft', 'liquidation', 'sandwich'];
    strategies.forEach(strategy => {
        const config = getConfiguration(strategy);
        if (config) {
            Object.keys(config).forEach(key => {
                const element = document.getElementById(`${strategy}-${key}`);
                if (element) {
                    element.value = config[key];
                }
            });
        }
    });
};


