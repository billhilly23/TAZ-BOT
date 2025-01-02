#!/bin/bash

# Check if a network was provided as an argument, otherwise default to rinkeby
NETWORK=${1:-rinkeby}
echo "Deploying contracts on network: $NETWORK"

# Create logs directory if it doesn't exist
mkdir -p logs

# Function to deploy a contract in parallel
deploy_contract() {
  CONTRACT_NAME=$1
  echo "Deploying $CONTRACT_NAME..."

  # Run the deployment in the background and log output
  npx hardhat run --network $NETWORK ./scripts/deploy_$CONTRACT_NAME.js > "logs/$CONTRACT_NAME.log" 2>&1 &

  if [ $? -eq 0 ]; then
    echo "$CONTRACT_NAME deployed successfully."
  else
    echo "Failed to deploy $CONTRACT_NAME. Check logs/$CONTRACT_NAME.log for details."
    exit 1
  fi
}

# Loop over all deploy scripts in the ./scripts directory
for deploy_script in ./scripts/deploy_*.js; do
  CONTRACT_NAME=$(basename "$deploy_script" .js | sed 's/deploy_//')
  deploy_contract "$CONTRACT_NAME"
done

# Wait for all background processes to finish
wait

echo "All contracts deployed successfully. Check 'logs/' for individual deployment details."

