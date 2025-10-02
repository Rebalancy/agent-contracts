// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";

import {IERC20Errors} from "@openzeppelin/interfaces/draft-IERC6093.sol";
import {ERC4626} from "@openzeppelin/token/ERC20/extensions/ERC4626.sol";

import {AaveVault} from "@src/AaveVault.sol";
import {MockUSDC} from "@src/mocks/MockUSDC.sol";
import {MockAavePool} from "@src/mocks/MockAavePool.sol";
import {MockAToken} from "@src/mocks/MockAToken.sol";

contract AaveVaultTest is Test {
    AaveVault public aaveVault;
    MockUSDC public mockUSDC;
    MockAavePool public mockAavePool;
    MockAToken public mockAToken;
    address public constant DEPLOYER = address(0xABCD);
    address public constant AI_AGENT = address(0x1234);
    string public constant VAULT_NAME = "Aave Vault";
    string public constant VAULT_SYMBOL = "AAVE-VLT";

    address public constant ALICE = address(0x9ABC);
    address public constant BOB = address(0x5678);
    address public constant CHARLIE = address(0xDEF0);

    function setUp() public {
        vm.startPrank(DEPLOYER);
        vm.deal(DEPLOYER, 100 ether);

        mockUSDC = new MockUSDC();
        mockAToken = new MockAToken();
        mockAavePool = new MockAavePool(mockUSDC, mockAToken);
        aaveVault = new AaveVault(mockUSDC, AI_AGENT, mockAavePool, mockAToken, VAULT_NAME, VAULT_SYMBOL);

        vm.stopPrank();
    }

    function testInitialSetup() public view {
        assertEq(address(aaveVault.asset()), address(mockUSDC));
        assertEq(aaveVault.name(), VAULT_NAME);
        assertEq(aaveVault.symbol(), VAULT_SYMBOL);
        assertEq(aaveVault.AI_AGENT(), AI_AGENT);
        assertEq(address(aaveVault.AAVE_POOL()), address(mockAavePool));
        assertEq(address(aaveVault.A_TOKEN()), address(mockAToken));
        assertEq(aaveVault.MAX_TOTAL_DEPOSITS(), 100_000_000 * 1e6); // 100M
        assertEq(aaveVault.crossChainInvestedAssets(), 0);
        assertEq(aaveVault.decimals(), 6);
        assertEq(aaveVault.totalSupply(), 0);
        assertEq(aaveVault.totalAssets(), 0);
        assertEq(aaveVault.asset(), address(mockUSDC));
    }

    function testDeposit() public {
        uint256 depositAmount = 100e6;

        mockUSDC.mint(ALICE, depositAmount);

        vm.startPrank(ALICE);

        mockUSDC.approve(address(aaveVault), depositAmount);

        uint256 shares = aaveVault.deposit(depositAmount, ALICE);

        assertEq(shares, depositAmount);
        assertEq(aaveVault.totalSupply(), shares);
        assertEq(aaveVault.totalAssets(), depositAmount);
        assertEq(mockUSDC.balanceOf(ALICE), 0);

        vm.stopPrank();
    }

    function testSingleDepositWithdraw(uint128 amount) public {
        vm.assume(amount > 1);
        vm.assume(amount < 100_000_000 * 1e6); // Avoid overflow issues with large amounts.

        uint256 aliceMockUSDCDeposit = amount;

        mockUSDC.mint(ALICE, aliceMockUSDCDeposit);

        vm.startPrank(ALICE);

        mockUSDC.approve(address(aaveVault), aliceMockUSDCDeposit);

        assertEq(mockUSDC.allowance(ALICE, address(aaveVault)), aliceMockUSDCDeposit);

        uint256 alicePreDepositBal = mockUSDC.balanceOf(ALICE);
        uint256 aliceShareAmount = aaveVault.deposit(aliceMockUSDCDeposit, ALICE);

        // Expect exchange rate to be 1:1 on initial mint.
        assertEq(aliceMockUSDCDeposit, aliceShareAmount);
        assertEq(aaveVault.previewWithdraw(aliceShareAmount), aliceMockUSDCDeposit);
        assertEq(aaveVault.previewDeposit(aliceMockUSDCDeposit), aliceShareAmount);
        assertEq(aaveVault.totalSupply(), aliceShareAmount);
        assertEq(aaveVault.totalAssets(), aliceMockUSDCDeposit);
        assertEq(aaveVault.balanceOf(ALICE), aliceShareAmount);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(ALICE)), aliceMockUSDCDeposit);
        assertEq(mockUSDC.balanceOf(ALICE), alicePreDepositBal - aliceMockUSDCDeposit);

        // withdraw
        aaveVault.withdraw(aliceMockUSDCDeposit, ALICE, ALICE);

        assertEq(aaveVault.totalAssets(), 0);
        assertEq(aaveVault.balanceOf(ALICE), 0);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(ALICE)), 0);
        assertEq(mockUSDC.balanceOf(ALICE), alicePreDepositBal);

        vm.stopPrank();
    }

    function testSingleMintRedeem(uint128 amount) public {
        vm.assume(amount > 1);
        vm.assume(amount < 100_000_000 * 1e6); // Avoid overflow issues with large amounts.

        uint256 aliceShareAmount = amount;

        mockUSDC.mint(ALICE, aliceShareAmount);

        vm.startPrank(ALICE);

        mockUSDC.approve(address(aaveVault), aliceShareAmount);
        assertEq(mockUSDC.allowance(ALICE, address(aaveVault)), aliceShareAmount);

        uint256 alicePreDepositBal = mockUSDC.balanceOf(ALICE);

        uint256 aliceUnderlyingAmount = aaveVault.mint(aliceShareAmount, ALICE);

        // Expect exchange rate to be 1:1 on initial mint.
        assertEq(aliceShareAmount, aliceUnderlyingAmount);
        assertEq(aaveVault.previewWithdraw(aliceShareAmount), aliceUnderlyingAmount);
        assertEq(aaveVault.previewDeposit(aliceUnderlyingAmount), aliceShareAmount);
        assertEq(aaveVault.totalSupply(), aliceShareAmount);
        assertEq(aaveVault.totalAssets(), aliceUnderlyingAmount);
        assertEq(aaveVault.balanceOf(ALICE), aliceUnderlyingAmount);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(ALICE)), aliceUnderlyingAmount);
        assertEq(mockUSDC.balanceOf(ALICE), alicePreDepositBal - aliceUnderlyingAmount);

        aaveVault.redeem(aliceShareAmount, ALICE, ALICE);

        assertEq(aaveVault.totalAssets(), 0);
        assertEq(aaveVault.balanceOf(ALICE), 0);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(ALICE)), 0);
        assertEq(mockUSDC.balanceOf(ALICE), alicePreDepositBal);

        vm.stopPrank();
    }

    function test_RevertWhen_DepositWithNotEnoughApproval() public {
        mockUSDC.mint(ALICE, 0.5e6);

        vm.startPrank(ALICE);

        mockUSDC.approve(address(aaveVault), 0.5e6);

        assertEq(mockUSDC.allowance(ALICE, address(aaveVault)), 0.5e6);

        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20Errors.ERC20InsufficientAllowance.selector,
                address(aaveVault), // spender
                0.5e6, // allowance (since no approval)
                1e6 // needed
            )
        );

        aaveVault.deposit(1e6, address(this));

        vm.stopPrank();
    }

    function test_RevertWhen_WithdrawWithNotEnoughUnderlyingAmount() public {
        mockUSDC.mint(ALICE, 0.5e6);

        vm.startPrank(ALICE);

        mockUSDC.approve(address(aaveVault), 0.5e6);

        aaveVault.deposit(0.5e6, ALICE);

        vm.expectRevert(
            abi.encodeWithSelector(
                ERC4626.ERC4626ExceededMaxWithdraw.selector,
                ALICE, // owner
                1e6, // amount to withdraw
                0.5e6 // available (only share balance)
            )
        );

        aaveVault.withdraw(1e6, ALICE, ALICE);

        vm.stopPrank();
    }

    function test_RevertWhen_RedeemWithNotEnoughShareAmount() public {
        mockUSDC.mint(ALICE, 0.5e6);

        vm.startPrank(ALICE);

        mockUSDC.approve(address(aaveVault), 0.5e6);

        aaveVault.deposit(0.5e6, ALICE);

        vm.expectRevert(
            abi.encodeWithSelector(
                ERC4626.ERC4626ExceededMaxRedeem.selector,
                ALICE, // owner
                1e6, // amount to redeem
                0.5e6 // available (only share balance)
            )
        );

        aaveVault.redeem(1e6, ALICE, ALICE);

        vm.stopPrank();
    }

    function test_RevertWhen_WithdrawWithNoUnderlyingAmount() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                ERC4626.ERC4626ExceededMaxWithdraw.selector,
                ALICE, // owner
                1e6, // amount to withdraw
                0 // available (since no share balance)
            )
        );

        vm.startPrank(ALICE);

        aaveVault.withdraw(1e6, ALICE, ALICE);

        vm.stopPrank();
    }

    function test_RevertWhen_RedeemWithNoShareAmount() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                ERC4626.ERC4626ExceededMaxRedeem.selector,
                ALICE, // owner
                1e6, // amount to redeem
                0 // available (since no share balance)
            )
        );

        vm.startPrank(ALICE);

        aaveVault.redeem(1e6, ALICE, ALICE);

        vm.stopPrank();
    }

    function test_RevertWhen_DepositWithNoApproval() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20Errors.ERC20InsufficientAllowance.selector,
                address(aaveVault), // spender
                0, // allowance (since no approval)
                1e6 // needed
            )
        );

        vm.startPrank(ALICE);

        aaveVault.deposit(1e6, ALICE);

        vm.stopPrank();
    }

    function test_RevertWhen_MintWithNoApproval() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20Errors.ERC20InsufficientAllowance.selector,
                address(aaveVault), // spender
                0, // allowance (since no approval)
                1e6 // needed
            )
        );

        vm.startPrank(ALICE);

        aaveVault.mint(1e6, ALICE);

        vm.stopPrank();
    }

    function test_DepositZero() public {
        vm.startPrank(ALICE);

        aaveVault.deposit(0, address(this));

        vm.stopPrank();

        assertEq(aaveVault.balanceOf(ALICE), 0);
    }

    function test_Mint_Zero() public {
        aaveVault.mint(0, address(this));

        assertEq(aaveVault.balanceOf(address(this)), 0);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(address(this))), 0);
        assertEq(aaveVault.totalSupply(), 0);
        assertEq(aaveVault.totalAssets(), 0);
    }

    function test_RevertWhen_RedeemZero() public {
        aaveVault.redeem(0, address(this), address(this));
    }

    function test_Withdraw_Zero() public {
        aaveVault.withdraw(0, address(this), address(this));

        assertEq(aaveVault.balanceOf(address(this)), 0);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(address(this))), 0);
        assertEq(aaveVault.totalSupply(), 0);
        assertEq(aaveVault.totalAssets(), 0);
    }

    function testMultipleMintDepositRedeemWithdraw() public {
        // Scenario:
        // A = Alice, B = Bob
        //  ________________________________________________________
        // | Vault shares | A share | A assets | B share | B assets |
        // |========================================================|
        // | 1. Alice mints 2000 shares (costs 2000 tokens)         |
        // |--------------|---------|----------|---------|----------|
        // |         2000 |    2000 |     2000 |       0 |        0 |
        // |--------------|---------|----------|---------|----------|
        // | 2. Bob deposits 4000 tokens (mints 4000 shares)        |
        // |--------------|---------|----------|---------|----------|
        // |         6000 |    2000 |     2000 |    4000 |     4000 |
        // |--------------|---------|----------|---------|----------|
        // | 3. Vault mutates by +3000 tokens...                    |
        // |    (simulated yield returned from AI Agent)...         |
        // |--------------|---------|----------|---------|----------|
        // |         6000 |    2000 |     3000 |    4000 |     6000 |
        // |--------------|---------|----------|---------|----------|
        // | 4. Alice deposits 2000 tokens (mints 1333 shares)      |
        // |--------------|---------|----------|---------|----------|
        // |         7333 |    3333 |     4999 |    4000 |     6000 |
        // |--------------|---------|----------|---------|----------|
        // | 5. Bob mints 2000 shares (costs 3001 assets)           |
        // |    NOTE: Bob's assets spent got rounded up             |
        // |    NOTE: Alice's vault assets got rounded up           |
        // |--------------|---------|----------|---------|----------|
        // |         9333 |    3333 |     5000 |    6000 |     9000 |
        // |--------------|---------|----------|---------|----------|
        // | 6. Vault mutates by +3000 tokens...                    |
        // |    (simulated yield returned from strategy)            |
        // |    NOTE: Vault holds 17001 tokens, but sum of          |
        // |          assetsOf() is 17000.                          |
        // |--------------|---------|----------|---------|----------|
        // |         9333 |    3333 |     6071 |    6000 |    10929 |
        // |--------------|---------|----------|---------|----------|
        // | 7. Alice redeem 1333 shares (2428 assets)              |
        // |--------------|---------|----------|---------|----------|
        // |         8000 |    2000 |     3643 |    6000 |    10929 |
        // |--------------|---------|----------|---------|----------|
        // | 8. Bob withdraws 2928 assets (1608 shares)             |
        // |--------------|---------|----------|---------|----------|
        // |         6392 |    2000 |     3643 |    4392 |     8000 |
        // |--------------|---------|----------|---------|----------|
        // | 9. Alice withdraws 3643 assets (2000 shares)           |
        // |    NOTE: Bob's assets have been rounded back up        |
        // |--------------|---------|----------|---------|----------|
        // |         4392 |       0 |        0 |    4392 |     8001 |
        // |--------------|---------|----------|---------|----------|
        // | 10. Bob redeem 4392 shares (8001 tokens)               |
        // |--------------|---------|----------|---------|----------|
        // |            0 |       0 |        0 |       0 |        0 |
        // |______________|_________|__________|_________|__________|

        mockUSDC.mint(ALICE, 4000);
        mockUSDC.mint(BOB, 7001);
        mockUSDC.mint(AI_AGENT, 10000);

        vm.startPrank(ALICE);
        mockUSDC.approve(address(aaveVault), 4000);
        assertEq(mockUSDC.allowance(ALICE, address(aaveVault)), 4000);
        vm.stopPrank();

        vm.startPrank(BOB);
        mockUSDC.approve(address(aaveVault), 7001);
        assertEq(mockUSDC.allowance(BOB, address(aaveVault)), 7001);
        vm.stopPrank();

        vm.startPrank(AI_AGENT);
        mockUSDC.approve(address(aaveVault), 10000);
        assertEq(mockUSDC.allowance(AI_AGENT, address(aaveVault)), 10000);
        vm.stopPrank();

        // 1. Alice mints 2000 shares (costs 2000 tokens)
        vm.startPrank(ALICE);
        uint256 aliceUnderlyingAmount = aaveVault.mint(2000, ALICE);
        uint256 aliceShareAmount = aaveVault.previewDeposit(aliceUnderlyingAmount);
        vm.stopPrank();

        // Expect to have received the requested mint amount.
        assertEq(aliceShareAmount, 2000);
        assertEq(aaveVault.balanceOf(ALICE), aliceShareAmount);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(ALICE)), aliceUnderlyingAmount);
        assertEq(aaveVault.convertToShares(aliceUnderlyingAmount), aaveVault.balanceOf(ALICE));

        // Expect a 1:1 ratio before mutation.
        assertEq(aliceUnderlyingAmount, 2000);

        // Sanity check.
        assertEq(aaveVault.totalSupply(), aliceShareAmount);
        assertEq(aaveVault.totalAssets(), aliceUnderlyingAmount);

        // 2. Bob deposits 4000 tokens (mints 4000 shares)
        vm.startPrank(BOB);

        uint256 bobShareAmount = aaveVault.deposit(4000, BOB);
        uint256 bobUnderlyingAmount = aaveVault.previewWithdraw(bobShareAmount);

        // Expect to have received the requested underlying amount.
        assertEq(bobUnderlyingAmount, 4000);
        assertEq(aaveVault.balanceOf(BOB), bobShareAmount);
        assertEq(aaveVault.convertToAssets(aaveVault.balanceOf(BOB)), bobUnderlyingAmount);
        assertEq(aaveVault.convertToShares(bobUnderlyingAmount), aaveVault.balanceOf(BOB));

        // Expect a 1:1 ratio before mutation.
        assertEq(bobShareAmount, bobUnderlyingAmount);

        // Sanity check.
        uint256 preMutationShareBal = aliceShareAmount + bobShareAmount;
        uint256 preMutationBal = aliceUnderlyingAmount + bobUnderlyingAmount;
        assertEq(aaveVault.totalSupply(), preMutationShareBal);
        assertEq(aaveVault.totalAssets(), preMutationBal);
        assertEq(aaveVault.totalSupply(), 6000);
        assertEq(aaveVault.totalAssets(), 6000);

        // 3. Vault earn by +3000 tokens (simulated yield deposited by the AI Agent)
        // The Vault now contains more tokens than deposited which causes the exchange rate to change.
        // Alice share is 33.33% of the Vault, Bob 66.66% of the Vault.
        // Alice's share count stays the same but the underlying amount changes from 2000 to 3000.
        // Bob's share count stays the same but the underlying amount changes from 4000 to 6000.
        uint256 mutationUnderlyingAmount = 3000;
        mockUSDC.mint(address(aaveVault), mutationUnderlyingAmount);
        assertEq(aaveVault.totalSupply(), preMutationShareBal);
        assertEq(aaveVault.totalAssets(), preMutationBal + mutationUnderlyingAmount);
        assertEq(aaveVault.balanceOf(ALICE), aliceShareAmount);
        //         assertEq(
        //             vault.convertToAssets(vault.balanceOf(alice)),
        //             aliceUnderlyingAmount + (mutationUnderlyingAmount / 3) * 1
        //         );
        //         assertEq(vault.balanceOf(bob), bobShareAmount);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), bobUnderlyingAmount + (mutationUnderlyingAmount / 3) * 2);

        //         // 4. Alice deposits 2000 tokens (mints 1333 shares)
        //         hevm.prank(alice);
        //         vault.deposit(2000, alice);

        //         assertEq(vault.totalSupply(), 7333);
        //         assertEq(vault.balanceOf(alice), 3333);
        //         assertEq(vault.convertToAssets(vault.balanceOf(alice)), 4999);
        //         assertEq(vault.balanceOf(bob), 4000);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), 6000);

        //         // 5. Bob mints 2000 shares (costs 3001 assets)
        //         // NOTE: Bob's assets spent got rounded up
        //         // NOTE: Alices's vault assets got rounded up
        //         hevm.prank(bob);
        //         vault.mint(2000, bob);

        //         assertEq(vault.totalSupply(), 9333);
        //         assertEq(vault.balanceOf(alice), 3333);
        //         assertEq(vault.convertToAssets(vault.balanceOf(alice)), 5000);
        //         assertEq(vault.balanceOf(bob), 6000);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), 9000);

        //         // Sanity checks:
        //         // Alice and bob should have spent all their tokens now
        //         assertEq(underlying.balanceOf(alice), 0);
        //         assertEq(underlying.balanceOf(bob), 0);
        //         // Assets in vault: 4k (alice) + 7k (bob) + 3k (yield) + 1 (round up)
        //         assertEq(vault.totalAssets(), 14001);

        //         // 6. Vault mutates by +3000 tokens
        //         // NOTE: Vault holds 17001 tokens, but sum of assetsOf() is 17000.
        //         underlying.mint(address(vault), mutationUnderlyingAmount);
        //         assertEq(vault.totalAssets(), 17001);
        //         assertEq(vault.convertToAssets(vault.balanceOf(alice)), 6071);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), 10929);

        //         // 7. Alice redeem 1333 shares (2428 assets)
        //         hevm.prank(alice);
        //         vault.redeem(1333, alice, alice);

        //         assertEq(underlying.balanceOf(alice), 2428);
        //         assertEq(vault.totalSupply(), 8000);
        //         assertEq(vault.totalAssets(), 14573);
        //         assertEq(vault.balanceOf(alice), 2000);
        //         assertEq(vault.convertToAssets(vault.balanceOf(alice)), 3643);
        //         assertEq(vault.balanceOf(bob), 6000);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), 10929);

        //         // 8. Bob withdraws 2929 assets (1608 shares)
        //         hevm.prank(bob);
        //         vault.withdraw(2929, bob, bob);

        //         assertEq(underlying.balanceOf(bob), 2929);
        //         assertEq(vault.totalSupply(), 6392);
        //         assertEq(vault.totalAssets(), 11644);
        //         assertEq(vault.balanceOf(alice), 2000);
        //         assertEq(vault.convertToAssets(vault.balanceOf(alice)), 3643);
        //         assertEq(vault.balanceOf(bob), 4392);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), 8000);

        //         // 9. Alice withdraws 3643 assets (2000 shares)
        //         // NOTE: Bob's assets have been rounded back up
        //         hevm.prank(alice);
        //         vault.withdraw(3643, alice, alice);

        //         assertEq(underlying.balanceOf(alice), 6071);
        //         assertEq(vault.totalSupply(), 4392);
        //         assertEq(vault.totalAssets(), 8001);
        //         assertEq(vault.balanceOf(alice), 0);
        //         assertEq(vault.convertToAssets(vault.balanceOf(alice)), 0);
        //         assertEq(vault.balanceOf(bob), 4392);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), 8001);

        //         // 10. Bob redeem 4392 shares (8001 tokens)
        //         hevm.prank(bob);
        //         vault.redeem(4392, bob, bob);
        //         assertEq(underlying.balanceOf(bob), 10930);
        //         assertEq(vault.totalSupply(), 0);
        //         assertEq(vault.totalAssets(), 0);
        //         assertEq(vault.balanceOf(alice), 0);
        //         assertEq(vault.convertToAssets(vault.balanceOf(alice)), 0);
        //         assertEq(vault.balanceOf(bob), 0);
        //         assertEq(vault.convertToAssets(vault.balanceOf(bob)), 0);

        //         // Sanity check
        //         assertEq(underlying.balanceOf(address(vault)), 0);
    }

    //     function testVaultInteractionsForSomeoneElse() public {
    //         // init 2 users with a 1e18 balance
    //         address alice = address(0xABCD);
    //         address bob = address(0xDCBA);
    //         underlying.mint(alice, 1e18);
    //         underlying.mint(bob, 1e18);

    //         hevm.prank(alice);
    //         underlying.approve(address(vault), 1e18);

    //         hevm.prank(bob);
    //         underlying.approve(address(vault), 1e18);

    //         // alice deposits 1e18 for bob
    //         hevm.prank(alice);
    //         vault.deposit(1e18, bob);

    //         assertEq(vault.balanceOf(alice), 0);
    //         assertEq(vault.balanceOf(bob), 1e18);
    //         assertEq(underlying.balanceOf(alice), 0);

    //         // bob mint 1e18 for alice
    //         hevm.prank(bob);
    //         vault.mint(1e18, alice);
    //         assertEq(vault.balanceOf(alice), 1e18);
    //         assertEq(vault.balanceOf(bob), 1e18);
    //         assertEq(underlying.balanceOf(bob), 0);

    //         // alice redeem 1e18 for bob
    //         hevm.prank(alice);
    //         vault.redeem(1e18, bob, alice);

    //         assertEq(vault.balanceOf(alice), 0);
    //         assertEq(vault.balanceOf(bob), 1e18);
    //         assertEq(underlying.balanceOf(bob), 1e18);

    //         // bob withdraw 1e18 for alice
    //         hevm.prank(bob);
    //         vault.withdraw(1e18, alice, bob);

    //         assertEq(vault.balanceOf(alice), 0);
    //         assertEq(vault.balanceOf(bob), 0);
    //         assertEq(underlying.balanceOf(alice), 1e18);
    //     }
}
