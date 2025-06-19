run_agent:
    uv run main.py

build-contract:
    echo "Building contract..."
    cd contract && cargo build

test:
    echo "Running tests..."
    cd contract && cargo test