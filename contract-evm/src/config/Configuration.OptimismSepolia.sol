// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";

import {DeploymentUtils} from "@utils/DeploymentUtils.sol";

import {Configuration} from "./Configuration.sol";
import {Constants} from "@constants/Constants.sol";

library ConfigurationOptimismSepolia {
    using DeploymentUtils for Vm;

    function getConfig(Vm _vm) external view returns (Configuration.ConfigValues memory) {
        address agentAddress = _vm.envAddress("AGENT_ADDRESS");

        return Configuration.ConfigValues({
            UNDERLYING_TOKEN: 0x5fd84259d66Cd46123540766Be93DFE6D43130D7, // USDC address from deployment https://developers.circle.com/stablecoins/usdc-contract-addresses#testnet
            AGENT_ADDRESS: agentAddress,
            VAULT_NAME: Constants.VAULT_NAME,
            VAULT_SYMBOL: Constants.VAULT_SYMBOL,
            POOL_ADDRESS: 0xb50201558B00496A145fE76f7424749556E326D8, // https://github.com/bgd-labs/aave-address-book/blob/main/src/AaveV3OptimismSepolia.sol
            A_TOKEN_ADDRESS: 0xa818F1B57c201E092C4A2017A91815034326Efd1
        });
    }
}
