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

contract DepositScript is BaseScript {
    using DeployerUtils for Vm;
    using DeploymentUtils for Vm;

    constructor() {
        _loadConfiguration();
    }

    function run() public {
        console2.log("Deposit and Withdraw Script");

        vm.startBroadcast(deployer);

        address usdcAddress = config.UNDERLYING_TOKEN;
        address aaveVaultAddress = vm.loadDeploymentAddress(Constants.AAVE_VAULT);

        require(aaveVaultAddress != address(0), "AaveVault not deployed");

        IERC20Metadata usdcTestnet = IERC20Metadata(usdcAddress);
        AaveVault aaveVault = AaveVault(aaveVaultAddress);

        console2.log("USDC testnet Address: %s", address(usdcTestnet));
        console2.log("AaveVault Address: %s", address(aaveVault));

        // Approve the AaveVault to spend MockUSDC
        usdcTestnet.approve(address(aaveVault), type(uint256).max);

        console2.log("Approved AaveVault to spend USDC testnet");

        // Deposit 1000 USDC testnet into AaveVault
        uint256 depositAmount = 1 ** usdcTestnet.decimals();
        console2.log("Depositing %s USDC testnet into AaveVault", depositAmount);
        aaveVault.deposit(depositAmount, deployer);

        console2.log("Deposit successful");
        vm.stopBroadcast();
    }
}
