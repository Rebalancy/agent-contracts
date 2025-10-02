// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {ERC20} from "@openzeppelin/token/ERC20/ERC20.sol";

contract MockAToken is ERC20 {
    uint8 USDC_DECIMALS = 6;
    string USDC_NAME = "Aave Token";
    string USDC_SYMBOL = "AToken";

    constructor() ERC20(USDC_NAME, USDC_SYMBOL) {}

    function decimals() public view virtual override returns (uint8) {
        return USDC_DECIMALS;
    }

    function mint(address to, uint256 value) public virtual {
        _mint(to, value);
    }

    function burn(address from, uint256 value) public virtual {
        _burn(from, value);
    }
}
