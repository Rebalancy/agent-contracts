// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";

import {AaveVault} from "@src/AaveVault.sol";

contract AaveVaultForkTest is Test {
    AaveVault public aaveVault;

    // Addresses for Arbitrum Sepolia
    address usdc = 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d; // USDC address in Arbitrum Sepolia
    address aavePool = 0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff; // Aave V3 Arbitrum Sepolia Pool address
    address aToken = 0x460b97BD498E1157530AEb3086301d5225b91216; // Aave V3 Arbitrum Sepolia aUSDC address

    address internal agentAddress;

    function setUp() public {
        aaveVault = AaveVault(0x22e6F67ed4e177e75a89cc44445346A95C7aDdfd); // @dev already deployed on Arbitrum Sepolia at this address
        agentAddress = vm.envAddress("AGENT_ADDRESS");
    }

    function testInitialSetup() public view {
        assertEq(aaveVault.AI_AGENT(), agentAddress);
        assertEq(address(aaveVault.AAVE_POOL()), aavePool);
        assertEq(address(aaveVault.A_TOKEN()), aToken);
        assertEq(aaveVault.MAX_TOTAL_DEPOSITS(), 100_000_000 * 1e6); // 100M
        assertEq(aaveVault.crossChainInvestedAssets(), 0);
        assertEq(aaveVault.decimals(), 6);
        assertEq(aaveVault.totalSupply(), 1_000_000);
        assertEq(aaveVault.totalAssets(), 1_000_000);
        assertEq(aaveVault.asset(), usdc);
    }

    function test_withdrawForCrossChainAllocation() public {
        vm.startPrank(agentAddress);
        // agent withdraws shares to invest in another chain
        uint256 sharesToWithdraw = 1_000_000; // 1 USDC (6 dec)
        uint256 currentCrossChainBalance = 0;
        uint256 withdrawnAssets = aaveVault.withdrawForCrossChainAllocation(sharesToWithdraw, currentCrossChainBalance);
        vm.stopPrank();

        // asserts after withdrawal
        uint256 agentBalance = IERC20(usdc).balanceOf(agentAddress);
        assertEq(agentBalance, withdrawnAssets, "agent balance after withdrawal");
        assertEq(IERC20(usdc).balanceOf(address(aaveVault)), 0, "vault balance after withdrawal");
        assertEq(IERC20(aToken).balanceOf(address(aaveVault)), 0, "atoken vault balance after withdrawal");
        assertEq(aaveVault.totalSupply(), withdrawnAssets, "totalSupply should be 1 after withdrawal");
        assertEq(aaveVault.totalAssets(), withdrawnAssets, "totalAssets should be 1 after withdrawal");
    }
}
