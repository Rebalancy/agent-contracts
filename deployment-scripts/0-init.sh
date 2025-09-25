#!/usr/bin/env bash
set -euo pipefail

DEST_DIR="solidity-contracts"
REPOS=(
  "https://github.com/Rebalancy/contracts.git"
)

mkdir -p "$DEST_DIR"
cd "$DEST_DIR"

for REPO in "${REPOS[@]}"; do
  NAME=$(basename "$REPO" .git)
  if [ -d "$NAME" ]; then
    echo "⚠️  Repo $NAME already exists, skipping..."
  else
    echo "➡️  Cloning $NAME..."
    git clone "$REPO"
  fi
done

echo "✅ All repositories initialized under $DEST_DIR/"