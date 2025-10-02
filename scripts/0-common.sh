#!/usr/bin/env bash
set -euo pipefail

update_env_var() {
  local key=$1
  local value=$2
  local file=$3

  if grep -q "^$key=" "$file"; then
    # macOS usa -i ''
    if [[ "$OSTYPE" == "darwin"* ]]; then
      sed -i '' "s|^$key=.*|$key=$value|" "$file"
    else
      sed -i "s|^$key=.*|$key=$value|" "$file"
    fi
    echo "🔄 Updated $key in $file"
  else
    echo "$key=$value" >> "$file"
    echo "➕ Added $key=$value to $file"
  fi
}