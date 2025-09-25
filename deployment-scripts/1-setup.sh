#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(pwd)

# initialise Solidity contracts
pushd "$ROOT_DIR/repos/contracts"
forge soldeer install   
forge compile         
popd

# initialise Python agent
pushd "$ROOT_DIR/repos/agent-contracts/agent"
uv sync                
popd

# initialise NEAR contract (Rust)
cd "$ROOT_DIR/repos/agent-contracts/contract"
echo "Inside $(pwd)"
cargo near build reproducible-wasm --variant force_bulk_memory


echo "✅ Setup complete"