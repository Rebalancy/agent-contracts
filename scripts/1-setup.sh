#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_SOLIDITY="$ROOT_DIR/solidity-contracts"
DEST_AGENT="$ROOT_DIR/agent"
DEST_CONTRACT="$ROOT_DIR/contract"

# initialise Solidity contracts
if [ -d "$DEST_SOLIDITY" ]; then
  pushd "$DEST_SOLIDITY" >/dev/null
  forge soldeer install
  forge compile
  popd >/dev/null
else
  echo "❌ Solidity contracts folder not found at $DEST_SOLIDITY"
fi

# initialise Python agent
if [ -d "$DEST_AGENT" ]; then
  pushd "$DEST_AGENT" >/dev/null
  uv sync
  popd >/dev/null
else
  echo "❌ Python agent folder not found at $DEST_AGENT"
fi

# initialise NEAR contract (Rust)
if [ -d "$DEST_CONTRACT" ]; then
  pushd "$DEST_CONTRACT" >/dev/null
  echo "📦 Building NEAR contract inside $(pwd)"
  cargo near build reproducible-wasm --variant force_bulk_memory
  popd >/dev/null
else
  echo "❌ NEAR contract folder not found at $DEST_CONTRACT"
fi

echo "✅ Setup complete"