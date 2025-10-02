run_agent:
    uv run main.py

build-contract:
    echo "Building contract..."
    cd contract && cargo near build non-reproducible-wasm

test:
    echo "Running tests..."
    cd contract && cargo test -- --nocapture

build-with-docker:
    cd contract && cargo near build reproducible-wasm --variant force_bulk_memory   

test-agent:
    echo "Running agent tests..."
    cd agent && PYTHONPATH=src uv run pytest

# contract-evm
compile-evm-contracts:
    echo "Compiling EVM contracts..."
    cd contract-evm && forge build

setup-evm-contracts:
    echo "Setting up EVM contracts..."
    cd contract-evm && forge soldeer install

test-evm-contracts-unit:
    echo "Running EVM unit tests..."
    cd contract-evm && just test_unit

test-evm-contracts-fork:
    echo "Running EVM fork tests..."
    cd contract-evm && just test_fork

