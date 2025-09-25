#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="$ROOT_DIR/solidity-contracts"

if [ -d "$DEST_DIR/.git" ]; then
  echo "🔄 Updating $DEST_DIR..."
  (cd "$DEST_DIR" && git pull --ff-only)
  echo "✅ Repo in $DEST_DIR updated"
else
  echo "❌ $DEST_DIR is not a git repository"
  exit 1
fi