// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IAavePool} from "../interfaces/IAavePool.sol";
import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";
import {MockAToken} from "./MockAToken.sol";

contract MockAavePool is IAavePool {
    IERC20 public immutable underlying;
    MockAToken public immutable aToken;

    constructor(IERC20 _underlying, MockAToken _aToken) {
        underlying = _underlying;
        aToken = _aToken;
    }

    function supply(address asset, uint256 amount, address onBehalfOf, uint16) external {
        require(asset == address(underlying), "bad-asset");
        // Pull the underlying asset from the caller
        IERC20(underlying).transferFrom(msg.sender, address(this), amount);
        // Mint aToken to the specified address
        aToken.mint(onBehalfOf, amount);
    }

    function withdraw(address asset, uint256 amount, address to) external returns (uint256) {
        require(asset == address(underlying), "bad-asset");
        // Burn aToken from the caller
        aToken.burn(msg.sender, amount);
        // Send the underlying asset to the specified address
        IERC20(underlying).transfer(to, amount);
        return amount;
    }
}
