run_agent:
    uv run main.py

build-contract:
    echo "Building contract..."
    cd contract && cargo near build non-reproducible-wasm

test:
    echo "Running tests..."
    cd contract && cargo test