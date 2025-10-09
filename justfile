set dotenv-load
set export

build-contract:
    echo "Building contract..."
    cd contract && cargo build

contract-run-init:
    echo "Running contract initialization test..."
    cd contract && cargo test --test test_init -- --nocapture
    
contract-run-test:
    echo "Running tests..."
    cd contract && cargo test -- --nocapture

build-with-docker:
    cd contract && cargo near build reproducible-wasm --variant force_bulk_memory   

test-agent:
    echo "Running agent tests..."
    cd agent && PYTHONPATH=src uv run pytest

# agent
run_agent:
    cd agent && uv run src/main.py

# contract
create-account ACCOUNT_ID:
    echo "Creating NEAR account: {{ACCOUNT_ID}}..."
    near create-account "{{ACCOUNT_ID}}" --useFaucet

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

test-evm-contracts-debug:
    echo "Running EVM debug tests..."
    cd contract-evm && just test_debug

deploy_arbitrum_sepolia:
    echo "Deploying to Arbitrum Sepolia..."
    cd contract-evm && just deploy_arbitrum_sepolia

seed_agent_address_in_arbitrum_sepolia:
    echo "Seeding agent address with ETH..."
    cd contract-evm && just seed_agent_address_in_arbitrum_sepolia