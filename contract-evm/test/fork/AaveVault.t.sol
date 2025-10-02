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
    string public constant VAULT_NAME = "Aave Vault";
    string public constant VAULT_SYMBOL = "AAVE-VLT";

    address public constant ALICE = address(0x9ABC);
    address public constant BOB = address(0x5678);
    address public constant CHARLIE = address(0xDEF0);

    // Addresses for Arbitrum Sepolia
    address usdc = 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d; // USDC address in Arbitrum Sepolia
    address aavePool = 0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff; // Aave V3 Arbitrum Sepolia Pool address
    address aToken = 0x460b97BD498E1157530AEb3086301d5225b91216; // Aave V3 Arbitrum Sepolia aUSDC address

    string RPC_URL_ARBITRUM_SEPOLIA = vm.envString("RPC_URL_ARBITRUM_SEPOLIA");

    // EIP-712 constants
    string internal constant DOMAIN_NAME = "AaveVault";
    string internal constant DOMAIN_VERSION = "1";
    bytes32 internal constant EIP712_DOMAIN_TYPEHASH =
        keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");

    string internal constant SNAP_TYPEDEF =
        "CrossChainBalanceSnapshot(uint256 balance,uint256 nonce,uint256 deadline,uint256 assets,address receiver)";

    // private key and address for AI agent
    uint256 internal agentPrivaKey = uint256(keccak256("AI_AGENT_PK_SEED_V1"));
    address internal agentAddress = vm.addr(agentPrivaKey);

    function setUp() public {
        arbitrumSepoliaFork = vm.createFork(RPC_URL_ARBITRUM_SEPOLIA);

        vm.selectFork(arbitrumSepoliaFork);

        vm.startPrank(DEPLOYER);

        vm.deal(DEPLOYER, 100 ether);

        aaveVault =
            new AaveVault(IERC20(usdc), agentAddress, IAavePool(aavePool), IERC20(aToken), VAULT_NAME, VAULT_SYMBOL);

        vm.stopPrank();
    }

    function testInitialSetup() public {
        vm.selectFork(arbitrumSepoliaFork);
        assertEq(address(aaveVault.asset()), usdc);
        assertEq(aaveVault.name(), VAULT_NAME);
        assertEq(aaveVault.symbol(), VAULT_SYMBOL);
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

    function test_depositWithExtraInfoViaSignature() public {
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

        // agent invests the withdrawn assets in another chain and now we do a deposit with extra info via signature
        // we assume the agent has invested the assets in another chain and a user is depositing with extra info
        vm.startPrank(ALICE);
        // amount to deposit with extra info
        uint256 secondDepositAmount = 5_000_000; // 5 USDC (6 dec)
        // snapshot
        uint256 nonce = 0;
        uint256 deadline = block.timestamp + 1 days;
        uint256 crosschainBalance = withdrawnAssets; // the balance that was invested in another chain

        // digest EIP-712
        bytes32 ds = _domainSeparator(address(aaveVault));
        bytes32 digest = _digestSnapshot(ds, crosschainBalance, nonce, deadline, secondDepositAmount, ALICE);

        // sign the digest
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(agentPrivaKey, digest);
        bytes memory signature = abi.encodePacked(r, s, v);

        // struct for cross-chain balance snapshot
        AaveVault.CrossChainBalanceSnapshot memory snap = AaveVault.CrossChainBalanceSnapshot({
            balance: crosschainBalance,
            nonce: nonce,
            deadline: deadline,
            assets: secondDepositAmount,
            receiver: ALICE
        });

        // call
        uint256 secondDepositShares =
            aaveVault.depositWithExtraInfoViaSignature(secondDepositAmount, ALICE, snap, signature);
        vm.stopPrank();

        // asserts
        assertGt(secondDepositShares, 0, "shares > 0");
        assertEq(aaveVault.crossChainBalanceNonce(), 1, "nonce increment");
        assertEq(aaveVault.crossChainInvestedAssets(), crosschainBalance, "cross-chain balance updated");

        uint256 aTokenBalance = IERC20(aToken).balanceOf(address(aaveVault));
        assertEq(aTokenBalance, secondDepositAmount, "aToken balance should increase");

        assertEq(
            aaveVault.totalSupply(),
            secondDepositShares + crosschainBalance,
            "totalSupply == secondDepositShares + crosschainBalance"
        );
    }

    // ---------- helpers ----------

    function _domainSeparator(address verifyingContract) internal view returns (bytes32) {
        return keccak256(
            abi.encode(
                EIP712_DOMAIN_TYPEHASH,
                keccak256(bytes(DOMAIN_NAME)),
                keccak256(bytes(DOMAIN_VERSION)),
                block.chainid,
                verifyingContract
            )
        );
    }

    function _structHashSnapshot(uint256 balance, uint256 nonce, uint256 deadline, uint256 assets, address receiver)
        internal
        view
        returns (bytes32)
    {
        // valida que el typehash canónico coincida con el del contrato
        bytes32 expectedTypeHash = vm.eip712HashType(SNAP_TYPEDEF);
        assertEq(expectedTypeHash, aaveVault.SNAPSHOT_TYPEHASH(), "SNAPSHOT_TYPEHASH mismatch");

        return vm.eip712HashStruct(SNAP_TYPEDEF, abi.encode(balance, nonce, deadline, assets, receiver));
    }

    function _digestSnapshot(
        bytes32 domainSeparator,
        uint256 balance,
        uint256 nonce,
        uint256 deadline,
        uint256 assets,
        address receiver
    ) internal view returns (bytes32) {
        bytes32 structHash = _structHashSnapshot(balance, nonce, deadline, assets, receiver);
        return keccak256(abi.encodePacked("\x19\x01", domainSeparator, structHash));
    }
}
