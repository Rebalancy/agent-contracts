// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";

import {AaveVault} from "@src/AaveVault.sol";
import {IAavePool} from "@src/interfaces/IAavePool.sol";

contract AaveVaultForkTest is Test {
    AaveVault public aaveVault;

    uint256 arbitrumSepoliaFork;

    address public constant DEPLOYER = address(0xABCD);

    address public constant ALICE = address(0x9ABC);
    address public constant BOB = address(0x5678);
    address public constant CHARLIE = address(0xDEF0);

    // Addresses for Arbitrum Sepolia
    address usdc = 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d; // USDC address in Arbitrum Sepolia
    address aavePool = 0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff; // Aave V3 Arbitrum Sepolia Pool address
    address aToken = 0x460b97BD498E1157530AEb3086301d5225b91216; // Aave V3 Arbitrum Sepolia aUSDC address

    string RPC_URL_ARBITRUM_SEPOLIA = vm.envString("RPC_URL_ARBITRUM_SEPOLIA");

    // private key and address for AI agent
    uint256 internal agentPrivaKey = uint256(keccak256("AI_AGENT_PK_SEED_V1"));
    address internal agentAddress = vm.addr(agentPrivaKey);

    function setUp() public {
        arbitrumSepoliaFork = vm.createFork(RPC_URL_ARBITRUM_SEPOLIA);

        vm.selectFork(arbitrumSepoliaFork);

        vm.startPrank(DEPLOYER);

        vm.deal(DEPLOYER, 100 ether);

        aaveVault = AaveVault(0x8ff2C1C0BD5b8E3C6C0e1E3bd5e2fFE3Bd5E2FFe); // already deployed on Arbitrum Sepolia at this address

        vm.stopPrank();
    }

    function testInitialSetup() public {
        vm.selectFork(arbitrumSepoliaFork);
        assertEq(address(aaveVault.asset()), usdc);
        assertEq(aaveVault.AI_AGENT(), agentAddress);
        assertEq(address(aaveVault.AAVE_POOL()), aavePool);
        assertEq(address(aaveVault.A_TOKEN()), aToken);
        assertEq(aaveVault.MAX_TOTAL_DEPOSITS(), 100_000_000 * 1e6); // 100M
        assertEq(aaveVault.crossChainInvestedAssets(), 0);
        assertEq(aaveVault.decimals(), 6);
        assertEq(aaveVault.totalSupply(), 0);
        assertEq(aaveVault.totalAssets(), 0);
        assertEq(aaveVault.asset(), usdc);
    }

    function test_withdrawForCrossChainAllocation() public {
        vm.selectFork(arbitrumSepoliaFork);

        // fund Alice and approve the vault
        uint256 amountToFund = 10_000_000; // 1 USDC (6 dec)
        deal(usdc, ALICE, amountToFund);

        // we call the vault as Alice
        vm.startPrank(ALICE);

        IERC20(usdc).approve(address(aaveVault), amountToFund);

        // initial deposit using the vault deposit's function without extra info
        uint256 initialDepositAmount = 1_000_000; // 0.1 USDC (6 dec)
        uint256 shares = aaveVault.deposit(initialDepositAmount, ALICE);

        assertGt(shares, 0, "shares > 0");
        assertEq(aaveVault.totalSupply(), shares, "totalSupply == shares");
        assertEq(aaveVault.totalAssets(), initialDepositAmount, "totalAssets == amount");
        assertEq(aaveVault.balanceOf(ALICE), shares, "balanceOf(ALICE) == shares");
        assertEq(IERC20(usdc).balanceOf(ALICE), amountToFund - initialDepositAmount, "ALICE balance after deposit");
        assertEq(IERC20(usdc).balanceOf(address(aaveVault)), 0, "vault balance after deposit");
        assertEq(
            IERC20(aToken).balanceOf(address(aaveVault)), initialDepositAmount, "atoken vault balance after deposit"
        );
        assertEq(
            aaveVault.convertToAssets(shares), initialDepositAmount, "convertToAssets(shares) == initialDepositAmount"
        );
        assertEq(aaveVault.crossChainBalanceNonce(), 0, "crossChainBalanceNonce should be 0");
        assertEq(aaveVault.crossChainInvestedAssets(), 0, "crossChainInvestedAssets should be 0");

        // the agent withdraws the shares to invest in another chain
        vm.stopPrank();
        vm.startPrank(agentAddress);
        // agent withdraws shares to invest in another chain
        uint256 sharesToWithdraw = shares;
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
