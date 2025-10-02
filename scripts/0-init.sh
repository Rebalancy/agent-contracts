#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="$ROOT_DIR/solidity-contracts"
REPO="https://github.com/Rebalancy/contracts.git"

if [ -d "$DEST_DIR/.git" ]; then
  echo "⚠️ Repo already exists at $DEST_DIR, skipping..."
else
  echo "➡️ Cloning into $DEST_DIR..."
  git clone "$REPO" "$DEST_DIR"
fi

echo "✅ Repo initialized at $DEST_DIR/"