// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";

import {DeploymentUtils} from "@utils/DeploymentUtils.sol";

import {Configuration} from "./Configuration.sol";
import {Constants} from "@constants/Constants.sol";

library ConfigurationLocal {
    using DeploymentUtils for Vm;

    function getConfig(Vm _vm) external view returns (Configuration.ConfigValues memory) {
        address mockUSDC = _vm.loadDeploymentAddress("MockUSDC");
        require(mockUSDC != address(0), "MockUSDC not deployed");

        return Configuration.ConfigValues({
            UNDERLYING_TOKEN: mockUSDC, // Mock USDC address from deployment
            AGENT_ADDRESS: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266,
            VAULT_NAME: Constants.VAULT_NAME,
            VAULT_SYMBOL: Constants.VAULT_SYMBOL,
            POOL_ADDRESS: Constants.ADDRESS_ZERO,
            A_TOKEN_ADDRESS: Constants.ADDRESS_ZERO
        });
    }
}
