#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const config = JSON.parse(fs.readFileSync("./config.json", "utf8"));
const zeroAddress = "0x0000000000000000000000000000000000000000";

function getVaultAddress(chainId) {
    console.error(`🔍 Reading vault for chain ${chainId}`);
    try {
        const filePath = path.join(
            __dirname,
            "..",
            "contract-evm",
            "deployments",
            String(chainId),
            "AaveVault.json"
        );
        const data = JSON.parse(fs.readFileSync(filePath, "utf8"));
        console.error(`✅ Found vault ${data.address}`);
        return data.address || zeroAddress;
    } catch {
        console.error(`⚠️ No vault for chain ${chainId}`);
        return zeroAddress;
    }
}

const initArgs = {
    source_chain: config.sourceChain,
    configs: Object.keys(config.chainIds).map((chainId) => {
        const cctp = config.cctpContracts[chainId] || {};
        const aave = config.aaveContracts[chainId] || {};

        return {
            chain_id: Number(chainId),
            config: {
                aave: {
                    asset: cctp.usdc || zeroAddress,
                    on_behalf_of: zeroAddress,
                    referral_code: 0,
                    lending_pool_address: aave.lendingPool || zeroAddress,
                },
                cctp: {
                    messenger_address: cctp.messenger || zeroAddress,
                    transmitter_address: cctp.transmitter || zeroAddress,
                },
                rebalancer: {
                    vault_address: config.vaultAddress || getVaultAddress(chainId),
                },
            },
        };
    }),
};

console.log(JSON.stringify(initArgs));