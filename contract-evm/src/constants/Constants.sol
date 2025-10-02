// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

library Constants {
    uint64 public constant ETHEREUM_MAINNET_NETWORK = 1;
    uint64 public constant ETHEREUM_SEPOLIA_NETWORK = 11155111;
    uint64 public constant BASE_MAINNET_NETWORK = 8453;
    uint64 public constant BASE_SEPOLIA_NETWORK = 84532;
    uint64 public constant ARBITRUM_MAINNET_NETWORK = 42161;
    uint64 public constant ARBITRUM_SEPOLIA_NETWORK = 421614;
    uint64 public constant OPTIMISM_MAINNET_NETWORK = 10;
    uint64 public constant OPTIMISM_SEPOLIA_NETWORK = 11155420;
    uint64 public constant LOCAL_NETWORK = 31337;
    uint64 public constant LOCAL_TEST_NETWORK = 3137;
    string public constant MOCK_USDC = "MockUSDC";
    string public constant AAVE_VAULT = "AaveVault";
    string public constant VAULT_NAME = "Aave Rebalancer";
    string public constant VAULT_SYMBOL = "AAVE-RB";
    address public constant ADDRESS_ZERO = address(0);
}
