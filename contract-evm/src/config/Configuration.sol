// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import "@openzeppelin/utils/Strings.sol";
import {Vm} from "forge-std/Vm.sol";

import {Constants} from "@constants/Constants.sol";

import {ConfigurationEthereumMainnet} from "./Configuration.Ethereum.sol";
import {ConfigurationEthereumSepolia} from "./Configuration.EthereumSepolia.sol";
import {ConfigurationBaseMainnet} from "./Configuration.Base.sol";
import {ConfigurationBaseSepolia} from "./Configuration.BaseSepolia.sol";
import {ConfigurationArbitrumSepolia} from "./Configuration.ArbitrumSepolia.sol";
import {ConfigurationLocal} from "./Configuration.Local.sol";

library Configuration {
    using Strings for uint64;

    struct ConfigValues {
        address UNDERLYING_TOKEN;
        address AGENT_ADDRESS;
        string VAULT_NAME;
        string VAULT_SYMBOL;
        address POOL_ADDRESS;
        address A_TOKEN_ADDRESS;
    }

    function load(Vm _vm) external view returns (ConfigValues memory) {
        uint64 chainId = uint64(block.chainid);

        if (chainId == Constants.LOCAL_NETWORK) {
            return ConfigurationLocal.getConfig(_vm);
        }

        if (chainId == Constants.ETHEREUM_MAINNET_NETWORK) {
            return ConfigurationEthereumMainnet.getConfig(_vm);
        }

        if (chainId == Constants.ETHEREUM_SEPOLIA_NETWORK) {
            return ConfigurationEthereumSepolia.getConfig(_vm);
        }

        if (chainId == Constants.BASE_MAINNET_NETWORK) {
            return ConfigurationBaseMainnet.getConfig(_vm);
        }

        if (chainId == Constants.BASE_SEPOLIA_NETWORK) {
            return ConfigurationBaseSepolia.getConfig(_vm);
        }

        if (chainId == Constants.ARBITRUM_SEPOLIA_NETWORK) {
            return ConfigurationArbitrumSepolia.getConfig(_vm);
        }

        revert(string(abi.encodePacked("Configuration: network not supported ", chainId.toString())));
    }
}
