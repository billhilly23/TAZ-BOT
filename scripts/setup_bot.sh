#!/bin/bash

# Load environment variables from .env file
if [ -f ".env" ]; then
  export $(grep -v '^#' .env | xargs)
else
  echo ".env file not found. Exiting."
  exit 1
fi

# Verify essential environment variables
missing_vars=()
[ -z "$INFURA_PROJECT_ID" ] && missing_vars+=("INFURA_PROJECT_ID")
[ -z "$ETHEREUM_NETWORK" ] && missing_vars+=("ETHEREUM_NETWORK")
[ -z "$WALLET_ADDRESS" ] && missing_vars+=("WALLET_ADDRESS")

if [ ${#missing_vars[@]} -gt 0 ]; then
  echo "The following environment variables are missing:"
  for var in "${missing_vars[@]}"; do
    echo "- $var"
  done
  echo "Please set them in your .env file."
  exit 1
fi

# Check if Cargo is installed
if ! command -v cargo &> /dev/null; then
  echo "Cargo could not be found. Please install Rust and Cargo."
  exit 1
fi

# Check if Cargo dependencies are resolved
if ! cargo check &> /dev/null; then
  echo "Cargo dependencies are not resolved. Please run 'cargo build' to ensure all dependencies are installed."
  exit 1
fi

# Run the bot
echo "Starting MEV bot on $ETHEREUM_NETWORK network..."
cargo run --release

