// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";

import {DeploymentUtils} from "@utils/DeploymentUtils.sol";

import {Configuration} from "./Configuration.sol";
import {Constants} from "@constants/Constants.sol";

library ConfigurationArbitrumSepolia {
    using DeploymentUtils for Vm;

    function getConfig(Vm _vm) external view returns (Configuration.ConfigValues memory) {
        address agentAddress = _vm.envAddress("AGENT_ADDRESS");

        return Configuration.ConfigValues({
            UNDERLYING_TOKEN: 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d, // USDC address from deployment https://developers.circle.com/stablecoins/usdc-contract-addresses#testnet
            AGENT_ADDRESS: agentAddress,
            VAULT_NAME: Constants.VAULT_NAME,
            VAULT_SYMBOL: Constants.VAULT_SYMBOL,
            POOL_ADDRESS: 0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff, // https://github.com/bgd-labs/aave-address-book/blob/main/src/AaveV3ArbitrumSepolia.sol
            A_TOKEN_ADDRESS: 0x460b97BD498E1157530AEb3086301d5225b91216
        });
    }
}
