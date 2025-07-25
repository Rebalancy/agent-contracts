run_agent:
    uv run main.py

build-contract:
    echo "Building contract..."
    cd contract && cargo near build non-reproducible-wasm

test:
    echo "Running tests..."
    cd contract && cargo test

just-build-with-docker:
    cargo near build reproducible-wasm --variant force_bulk_memory   