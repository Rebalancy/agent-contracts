// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {console2} from "forge-std/console2.sol";
import {Vm} from "forge-std/Vm.sol";

import {IERC20Metadata} from "@openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";

import {DeploymentUtils} from "@utils/DeploymentUtils.sol";
import {DeployerUtils} from "@utils/DeployerUtils.sol";
import {Constants} from "@constants/Constants.sol";

import {BaseScript} from "./BaseScript.s.sol";
import {AaveVault} from "../src/AaveVault.sol";

contract TransferUSDCScript is BaseScript {
    using DeployerUtils for Vm;
    using DeploymentUtils for Vm;

    uint256 public constant AMOUNT = 0.1 * 10 ** 6; // 0.1 USDC (6 decimals)

    constructor() {
        _loadConfiguration();
    }

    function run() public {
        console2.log("Transferring USDC...");
        address agentAddress = vm.envAddress("AGENT_ADDRESS");

        vm.startBroadcast(deployer);

        IERC20Metadata underlyingToken = IERC20Metadata(config.UNDERLYING_TOKEN);
        bool success = underlyingToken.transfer(agentAddress, AMOUNT);
        require(success, "USDC transfer failed");
        console2.log("Transfer successful");
        vm.stopBroadcast();
    }
}
