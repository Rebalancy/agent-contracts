// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";

import {DeploymentUtils} from "@utils/DeploymentUtils.sol";

import {Configuration} from "./Configuration.sol";

library ConfigurationEthereumSepolia {
    using DeploymentUtils for Vm;

    function getConfig(Vm) external pure returns (Configuration.ConfigValues memory) {
        revert("ConfigurationEthereumSepolia: not implemented");
    }
}
