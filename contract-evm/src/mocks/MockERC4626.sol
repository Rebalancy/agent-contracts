// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {ERC4626} from "@openzeppelin/token/ERC20/extensions/ERC4626.sol";
import {ERC20} from "@openzeppelin/token/ERC20/ERC20.sol";

contract MockERC4626 is ERC4626 {
    constructor(ERC20 _underlying) ERC4626(_underlying) ERC20("MockERC4626", "MockERC4626") {}
}
