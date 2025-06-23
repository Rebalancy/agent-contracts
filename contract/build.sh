#!/bin/bash
set -e

INPUT_WASM="target/near/shade_agent_contract.wasm"
TEMP_WASM="target/near/opt_temp.wasm"

echo "🛠 Compilando sin wasm-opt..."
cargo near build non-reproducible-wasm --no-wasmopt

echo "🔍 Verificando wasm-opt..."
if ! command -v wasm-opt &> /dev/null; then
    echo "❌ wasm-opt no está instalado. Instálalo con 'brew install binaryen'"
    exit 1
fi

echo "✨ Optimizing in-place with --enable-bulk-memory"
wasm-opt -O --enable-bulk-memory -o "$TEMP_WASM" "$INPUT_WASM"
mv "$TEMP_WASM" "$INPUT_WASM"

echo "📦 Generando ABI..."
cargo near abi

echo "✅ .wasm optimizado directamente en $INPUT_WASM"