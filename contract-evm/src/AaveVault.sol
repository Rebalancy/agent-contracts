// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {ERC4626} from "@openzeppelin/token/ERC20/extensions/ERC4626.sol";
import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";
import {ERC20} from "@openzeppelin/token/ERC20/ERC20.sol";
import {SafeERC20} from "@openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {Math} from "@openzeppelin/utils/math/Math.sol";
import {EIP712} from "@openzeppelin/utils/cryptography/EIP712.sol";
import {SignatureChecker} from "@openzeppelin/utils/cryptography/SignatureChecker.sol";
import {IERC20Metadata} from "@openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";
import {IAavePool} from "./interfaces/IAavePool.sol";
import {
    InvalidAmount,
    NotEnoughAssetsToInvest,
    InvalidAgentAddress,
    InvalidUnderlyingAddress,
    InvalidAavePoolAddress,
    InvalidATokenAddress,
    NotEnoughAssetsToWithdraw,
    NotEnoughLiquidity,
    SignatureExpired,
    BadNonce,
    InvalidSignature,
    InvalidAssetsAmount,
    InvalidReceiverAddress,
    InvalidAgentCaller,
    FutureDepositsRequireExtraInfoViaSignature,
    CannotReportCrossChainBalanceWhenNoAssetsInvested
} from "./Errors.sol";

contract AaveVault is ERC4626, EIP712 {
    using SafeERC20 for IERC20;
    using Math for uint256;

    /// @dev Address of the AI agent that will manage the investments.
    address public AI_AGENT; // TODO: This has to be immutable

    /// @dev Maximum total deposits allowed in the vault.
    uint256 public immutable MAX_TOTAL_DEPOSITS = 100_000_000 * 1e6; // 100M

    /// @dev Aave v3 Pool.
    IAavePool public immutable AAVE_POOL;

    /// @dev This is the token that represents the shares in the Aave pool.
    IERC20 public immutable A_TOKEN;

    uint256 public crossChainInvestedAssets;

    // ------- Snapshot EIP-712 -------
    struct CrossChainBalanceSnapshot {
        uint256 balance; // cross-chain balance in aTokens
        uint256 nonce; // anti-replay
        uint256 deadline; // expiration (unix timestamp)
        uint256 assets; // assets to deposit in the vault
        address receiver; // receiver of the shares
    }

    string private constant SIGNING_DOMAIN = "AaveVault";
    string private constant SIGNING_DOMAIN_VERSION = "1";

    bytes32 public constant SNAPSHOT_TYPEHASH = keccak256(
        "CrossChainBalanceSnapshot(uint256 balance,uint256 nonce,uint256 deadline,uint256 assets,address receiver)"
    );

    uint256 public crossChainBalanceNonce;

    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/
    event AssetsInvested(address indexed agent, uint256 amount, uint256 totalInvested);
    event CrossChainBalanceUpdated(uint256 aTokenBalanceBefore, uint256 aTokenBalanceAfter);
    event AutoInvested(uint256 amountInvested, uint256 aTokenBalanceAfter);
    event CrossChainFundsReturned(uint256 amountReturned, uint256 newCrossChainBalance, uint256 aTokenAfter);

    constructor(
        IERC20 _underlying,
        address _agentAddress,
        IAavePool _aavePool,
        IERC20 _aToken,
        string memory _name,
        string memory _symbol
    ) ERC4626(_underlying) ERC20(_name, _symbol) EIP712(SIGNING_DOMAIN, SIGNING_DOMAIN_VERSION) {
        if (_agentAddress == address(0)) {
            revert InvalidAgentAddress();
        }

        if (address(_underlying) == address(0)) {
            revert InvalidUnderlyingAddress();
        }

        if (address(_aavePool) == address(0)) {
            revert InvalidAavePoolAddress();
        }

        if (address(_aToken) == address(0)) {
            revert InvalidATokenAddress();
        }

        AI_AGENT = _agentAddress;
        AAVE_POOL = _aavePool;
        A_TOKEN = _aToken;

        // Approve the Aave pool to spend the underlying asset
        _underlying.approve(address(_aavePool), type(uint256).max);
    }

    /*//////////////////////////////////////////////////////////////
                        OVERRIDE FUNCTIONS
    //////////////////////////////////////////////////////////////*/
    function decimals() public view virtual override(ERC4626) returns (uint8) {
        return IERC20Metadata(asset()).decimals();
    }

    function totalAssets() public view virtual override returns (uint256) {
        uint256 assetsNotInvested = IERC20(asset()).balanceOf(address(this));
        uint256 assetsInvestedCrossChain = getCrossChainInvestedAssets();
        uint256 assetsInvestedInAave = A_TOKEN.balanceOf(address(this)); // 1 aToken ≈ 1 asset
        return assetsNotInvested + assetsInvestedInAave + assetsInvestedCrossChain;
    }

    /// @dev Deposit function that allows deposits only if there are no cross-chain invested assets.
    function deposit(uint256 _assets, address _receiver) public virtual override returns (uint256) {
        if (crossChainInvestedAssets > 0) {
            revert FutureDepositsRequireExtraInfoViaSignature();
        }
        return super.deposit(_assets, _receiver);
    }

    function depositWithExtraInfoViaSignature(
        uint256 _assets,
        address _receiver,
        CrossChainBalanceSnapshot calldata _snapshot,
        bytes calldata signature
    ) public virtual returns (uint256) {
        if (crossChainInvestedAssets == 0) {
            revert CannotReportCrossChainBalanceWhenNoAssetsInvested();
        }
        if (_receiver != msg.sender) {
            revert InvalidReceiverAddress();
        }

        // 1) Validate snapshot
        if (block.timestamp > _snapshot.deadline) {
            revert SignatureExpired();
        }
        if (_snapshot.balance == 0) {
            revert InvalidAmount();
        }
        if (_snapshot.nonce != crossChainBalanceNonce) {
            revert BadNonce();
        }

        if (_snapshot.assets != _assets) {
            revert InvalidAssetsAmount();
        }

        if (_snapshot.receiver != _receiver) {
            revert InvalidReceiverAddress();
        }

        // 2) Hash EIP-712 + verify signature
        bytes32 digest = _hashSnapshot(_snapshot);
        if (!SignatureChecker.isValidSignatureNow(AI_AGENT, digest, signature)) {
            revert InvalidSignature();
        }

        // 3) Increment nonce to prevent replay attacks
        unchecked {
            crossChainBalanceNonce++;
        }

        _updateCrossChainBalance(_snapshot.balance);

        // 4) Normal deposit logic
        uint256 maxAssets = maxDeposit(_receiver);
        if (_assets > maxAssets) {
            revert ERC4626ExceededMaxDeposit(_receiver, _assets, maxAssets);
        }

        uint256 shares = previewDeposit(_assets);
        _deposit(_msgSender(), _receiver, _assets, shares);

        return shares;
    }

    function maxDeposit(address) public view override returns (uint256) {
        uint256 ta = totalAssets();
        if (ta >= MAX_TOTAL_DEPOSITS) {
            return 0;
        }
        return MAX_TOTAL_DEPOSITS - ta;
    }

    function _deposit(address caller, address receiver, uint256 assets, uint256 shares) internal virtual override {
        super._deposit(caller, receiver, assets, shares);

        uint256 idleUnderlying = IERC20(asset()).balanceOf(address(this));
        if (idleUnderlying > 0) {
            AAVE_POOL.supply(address(asset()), idleUnderlying, address(this), 0); // referralCode = 0
            emit AutoInvested(idleUnderlying, A_TOKEN.balanceOf(address(this)));
        }
    }

    function _withdraw(address caller, address receiver, address owner, uint256 assets, uint256 shares)
        internal
        virtual
        override
    {
        uint256 idleUnderlying = IERC20(asset()).balanceOf(address(this));
        if (idleUnderlying < assets) {
            uint256 need = assets - idleUnderlying;
            uint256 got = AAVE_POOL.withdraw(address(asset()), need, address(this));
            if (idleUnderlying + got < assets) {
                revert NotEnoughAssetsToWithdraw();
            }
        }

        super._withdraw(caller, receiver, owner, assets, shares);
    }

    /*//////////////////////////////////////////////////////////////
                            AGENT FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    function withdrawForCrossChainAllocation(uint256 _amountToWithdraw, uint256 _crossChainATokenBalance)
        external
        onlyAgent
        returns (uint256)
    {
        if (_amountToWithdraw == 0) {
            revert InvalidAmount();
        }

        // Check if the vault has enough assets available
        if (_amountToWithdraw > getAvailableAssets()) {
            revert NotEnoughAssetsToInvest();
        }

        // Check the vault has enough aTokens
        if (_amountToWithdraw > A_TOKEN.balanceOf(address(this))) {
            revert NotEnoughLiquidity();
        }

        // Withdraw from AAVE
        uint256 amountWithdrawn = AAVE_POOL.withdraw(address(asset()), _amountToWithdraw, address(this));

        // Transfer the underlying asset to the agent for investment
        IERC20(asset()).safeTransfer(msg.sender, amountWithdrawn);

        crossChainInvestedAssets = amountWithdrawn + _crossChainATokenBalance;

        emit AssetsInvested(msg.sender, _amountToWithdraw, crossChainInvestedAssets);

        return amountWithdrawn;
    }

    function updateCrossChainBalance(uint256 _crossChainATokenBalance) external onlyAgent {
        _updateCrossChainBalance(_crossChainATokenBalance);
    }

    function _updateCrossChainBalance(uint256 _crossChainATokenBalance) internal {
        uint256 crosschainAssetsBefore = crossChainInvestedAssets;
        crossChainInvestedAssets = _crossChainATokenBalance;

        emit CrossChainBalanceUpdated(crosschainAssetsBefore, _crossChainATokenBalance);
    }

    function returnFunds(uint256 amountReturned, uint256 newCrossChainATokenBalance) external onlyAgent {
        if (amountReturned > 0) {
            IERC20(asset()).safeTransferFrom(msg.sender, address(this), amountReturned);
            AAVE_POOL.supply(address(asset()), amountReturned, address(this), 0);
        }

        _updateCrossChainBalance(newCrossChainATokenBalance);

        emit CrossChainFundsReturned(amountReturned, newCrossChainATokenBalance, A_TOKEN.balanceOf(address(this)));
    }

    /*//////////////////////////////////////////////////////////////
                         INTERNAL FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    function getAvailableAssets() internal view returns (uint256) {
        uint256 assetsNotInvested = IERC20(asset()).balanceOf(address(this));
        uint256 assetsInvestedInAave = A_TOKEN.balanceOf(address(this)); // 1 aToken ≈ 1 asset

        return assetsNotInvested + assetsInvestedInAave;
    }

    function getCrossChainInvestedAssets() internal view returns (uint256) {
        return crossChainInvestedAssets;
    }

    modifier onlyAgent() {
        // Check if the caller is the AI agent
        if (msg.sender != AI_AGENT) {
            revert InvalidAgentCaller();
        }
        _;
    }

    /*//////////////////////////////////////////////////////////////
                        EIP-712 SNAPSHOT FUNCTIONS
    //////////////////////////////////////////////////////////////*/
    function _hashSnapshot(CrossChainBalanceSnapshot memory _snapshot) internal view returns (bytes32) {
        return _hashTypedDataV4(
            keccak256(
                abi.encode(
                    SNAPSHOT_TYPEHASH,
                    _snapshot.balance,
                    _snapshot.nonce,
                    _snapshot.deadline,
                    _snapshot.assets,
                    _snapshot.receiver
                )
            )
        );
    }
}
