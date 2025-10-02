// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {console2} from "forge-std/console2.sol";
import {Vm} from "forge-std/Vm.sol";

import {DeploymentUtils} from "@utils/DeploymentUtils.sol";
import {DeployerUtils} from "@utils/DeployerUtils.sol";
import {Constants} from "@constants/Constants.sol";
import {MockUSDC} from "@mocks/MockUSDC.sol";
import {BaseScript} from "./BaseScript.s.sol";

contract MintMockUSDCScript is BaseScript {
    using DeployerUtils for Vm;
    using DeploymentUtils for Vm;

    MockUSDC public mockUSDC;

    constructor() {
        _loadConfiguration();
    }

    function run() public {
        console2.log("Minting USDC tokens");

        vm.startBroadcast(deployer);

        mockUSDC = MockUSDC(config.UNDERLYING_TOKEN);

        mockUSDC.mint(deployer, 100_000_000 * 10 ** 6); // Mint 1 million USDC

        vm.stopBroadcast();
    }
}
