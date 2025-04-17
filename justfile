run_agent:
    uv run main.py

build-contract:
    echo "Building contract..."
    cd contract && cargo build