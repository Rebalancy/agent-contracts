#!/usr/bin/env bash
set -euo pipefail

# load common functions and variables
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=0-common.sh
. "$SCRIPT_DIR/0-common.sh"

# 1) Create a new NEAR account with an incremented index

# base name for the accounts
BASE_NAME="rebalancer-abcdefgh"
NETWORK="testnet"
PATH_STR="ethereum-1"
DEPLOYMENTS_DIR="deployments"

# create deployments directory if it doesn't exist
if [ ! -d "$DEPLOYMENTS_DIR" ]; then
  echo "📂 Creating deployments folder..."
  mkdir -p "$DEPLOYMENTS_DIR"
fi

# find the last index used
if [ "$(ls -A $DEPLOYMENTS_DIR)" ]; then
  LAST_INDEX=$(jq -r '.index' $(ls -t $DEPLOYMENTS_DIR/deploy-*.json | head -n1))
else
  LAST_INDEX=0
fi

NEW_INDEX=$((LAST_INDEX + 1))
ACCOUNT_ID="${BASE_NAME}-${NEW_INDEX}.${NETWORK}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H-%M-%SZ")

# build the new account ID
ACCOUNT_ID="${BASE_NAME}-${NEW_INDEX}.${NETWORK}"

echo "➡️  Creating account: $ACCOUNT_ID"
near create-account "$ACCOUNT_ID" --useFaucet

echo "✅ Account $ACCOUNT_ID created and saved (index=$NEW_INDEX)"

# 2) Calculate the derived address for the agent 
EVMTARGET=$(cd agent && uv run python -m src.utils "$ACCOUNT_ID" "$NETWORK" "$PATH_STR")
echo "🔑 Derived EVM address for $ACCOUNT_ID: $EVMTARGET"

DEPLOY_FILE="$DEPLOYMENTS_DIR/deploy-$TIMESTAMP.json"

cat > "$DEPLOY_FILE" <<EOF
{
  "index": $NEW_INDEX,
  "account_id": "$ACCOUNT_ID",
  "network": "$NETWORK",
  "path_str": "$PATH_STR",
  "evm_address": "$EVMTARGET",
  "timestamp": "$TIMESTAMP"
}
EOF

echo "📦 Deployment saved at $DEPLOY_FILE"

# 3) Deploy the solidity contracts using the derived address
echo "🚀 Deploying solidity contracts using the derived address: $EVMTARGET"

# 4) Save the derived address into the .env file
update_env_var "AGENT_ADDRESS" "$EVMTARGET" ".env"

# 5) Deploy the evm contracts
just deploy_arbitrum_sepolia # @dev this includes initial deposit

# 6) Seed Agent Address
just seed_agent_address_in_arbitrum_sepolia
just seed_agent_address_in_optimism_sepolia

# 6) Deploy the agent contract to the new account
near deploy $ACCOUNT_ID "contract/target/near/shade_agent_contract.wasm" \
  --initFunction init \
  --initArgs "$(node scripts/gen-init-args.js 2>/dev/null)"

echo "✅ Agent contract deployed to $ACCOUNT_ID"

update_env_var NEAR_CONTRACT_ACCOUNT "$ACCOUNT_ID" ".env"

sleep 10

echo "✅ Deployment process completed successfully!"