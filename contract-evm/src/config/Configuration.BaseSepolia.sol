// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";

import {DeploymentUtils} from "@utils/DeploymentUtils.sol";

import {Configuration} from "./Configuration.sol";
import {Constants} from "@constants/Constants.sol";

library ConfigurationBaseSepolia {
    using DeploymentUtils for Vm;

    function getConfig(Vm) external pure returns (Configuration.ConfigValues memory) {
        return Configuration.ConfigValues({
            UNDERLYING_TOKEN: 0x036CbD53842c5426634e7929541eC2318f3dCF7e, // USDC address from deployment https://developers.circle.com/stablecoins/usdc-contract-addresses#testnet
            AGENT_ADDRESS: 0xD5aC5A88dd3F1FE5dcC3ac97B512Faeb48d06AF0, // AI Agent address
            VAULT_NAME: Constants.VAULT_NAME,
            VAULT_SYMBOL: Constants.VAULT_SYMBOL,
            POOL_ADDRESS: Constants.ADDRESS_ZERO,
            A_TOKEN_ADDRESS: Constants.ADDRESS_ZERO
        });
    }
}
