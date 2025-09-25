#!/usr/bin/env bash
set -euo pipefail

DEST_DIR="repositories"

cd "$DEST_DIR"

for DIR in */; do
  if [ -d "$DIR/.git" ]; then
    echo "🔄 Updating $DIR..."
    (cd "$DIR" && git pull origin main)
  else
    echo "⚠️  $DIR is not a git repository, skipping..."
  fi
done

echo "✅ All repositories in $DEST_DIR updated"