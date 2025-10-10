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

contract TransferETHScript is BaseScript {
    using DeployerUtils for Vm;
    using DeploymentUtils for Vm;

    uint256 public constant AMOUNT = 0.1 ether;

    constructor() {
        _loadConfiguration();
    }

    function run() public {
        console2.log("Transferring ETH...");
        address agentAddress = vm.envAddress("AGENT_ADDRESS");

        vm.startBroadcast(deployer);

        (bool success,) = payable(agentAddress).call{value: AMOUNT}("");
        require(success, "ETH transfer failed");
        console2.log("Transfer successful");
        vm.stopBroadcast();
    }
}
