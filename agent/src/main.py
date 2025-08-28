import os
import asyncio
import json

from dotenv import load_dotenv

from near_omni_client.providers.near import NearFactoryProvider
from near_omni_client.networks import Network
from near_omni_client.providers.evm import AlchemyFactoryProvider
from near_omni_client.wallets import MPCWallet
from near_omni_client.wallets.near_wallet import NearWallet
from near_omni_client.crypto.keypair import KeyPair
from near_omni_client.providers.near import NearFactoryProvider
from near_omni_client.json_rpc.client import NearClient
from near_omni_client.chain_signatures.kdf import Kdf
from near_omni_client.chain_signatures.utils import get_evm_address
from vault_abi import get_vault_abi
from utils import from_chain_id_to_network, parse_chain_balances, parse_chain_configs, parse_u32_result, to_usdc_units
from optimizer_data_fetcher import get_extra_data_for_optimization
from optimizer import optimize_chain_allocation_with_direction
from rebalancer import compute_rebalance_operations, execute_all_rebalances

PATH = "rebalancer.testnet"

async def main():
    load_dotenv() 
    contract_id = os.getenv("NEAR_CONTRACT_ACCOUNT", "rebalancer.testnet")
    near_network = os.getenv("NEAR_NETWORK", "testnet")
    alchemy_api_key = os.getenv("ALCHEMY_API_KEY", "your_alchemy_api_key_here")
    max_bridge_fee = float(os.getenv("MAX_BRIDGE_FEE", "0.99"))
    min_bridge_finality_threshold = int(os.getenv("MIN_BRIDGE_FINALITY_THRESHOLD", "1000"))

    one_time_signer_private_key = os.getenv("ONE_TIME_SIGNER_PRIVATE_KEY", "your_private_key_here")
    one_time_signer_account_id = os.getenv("ONE_TIME_SIGNER_ACCOUNT_ID", "your_account_id_here")
    one_time_signer_public_key = os.getenv("ONE_TIME_SIGNER_PUBLIC_KEY", "your_public_key_here")

    # print("one_time_signer_private_key:", one_time_signer_private_key)
    # print("one_time_signer_account_id:", one_time_signer_account_id)
    # print("one_time_signer_public_key:", one_time_signer_public_key)

    near_network = Network.parse(near_network)
    near_factory_provider = NearFactoryProvider()
    near_client = near_factory_provider.get_provider(near_network)
    alchemy_factory_provider = AlchemyFactoryProvider(api_key=alchemy_api_key)
    chain_config_raw = await near_client.call_contract(
        contract_id=contract_id,
        method="get_all_configs",
        args={}
    )
    configs = parse_chain_configs(chain_config_raw)
    
    # print("Parsed Chain Configs:", configs)

    source_chain_raw = await near_client.call_contract(
        contract_id=contract_id,
        method="get_source_chain",
        args={}
    )
    source_chain_id = parse_u32_result(source_chain_raw)

    # print("Source Chain ID:", source_chain_id)

    current_allocations_raw = await near_client.call_contract(
        contract_id=contract_id,
        method="get_allocations",
        args={}
    )

    current_allocations = parse_chain_balances(current_allocations_raw)
    print("Current Allocations:", current_allocations)
    
    source_chain_config = configs.get(source_chain_id, None)
    if not source_chain_config:
        raise ValueError(f"Source chain config for chain ID {source_chain_id} not found in configs.")

    vault_address = source_chain_config["rebalancer"]["vault_address"]
    if not vault_address:
        raise ValueError(f"Vault address for source chain {source_chain_id} not found in configs.")

    mpc_wallet = MPCWallet(
        path=PATH,
        account_id=contract_id, # The account ID of the contract on NEAR
        near_network=near_network,
        provider_factory=alchemy_factory_provider,
        supported_networks=[Network.OPTIMISM_SEPOLIA, Network.ARBITRUM_SEPOLIA],
    )

    if all(v == 0 for v in current_allocations.values()):
        print("⚠️ No prior rebalance detected. Fetching totalAssets from source chain...")
        
        web3 = mpc_wallet.get_web3(from_chain_id_to_network(source_chain_id))
        contract = web3.eth.contract(address=vault_address, abi=get_vault_abi())
        total_assets_under_management = contract.functions.totalAssets().call()
        print(f"🔁 Using Total Assets Under Management from source chain ({source_chain_id}): {total_assets_under_management}")

        current_allocations[source_chain_id] = total_assets_under_management

    print("Current Allocations after fetching totalAssets:", current_allocations)
    
    override_rates= os.getenv("OVERRIDE_INTEREST_RATES", "{}")
    try:
        override_interest_rates_raw = json.loads(override_rates)
        override_interest_rates = {int(k): v for k, v in override_interest_rates_raw.items()}
    except json.JSONDecodeError:
        raise ValueError("OVERRIDE_INTEREST_RATES must be a valid JSON dictionary.")

    # print("Override Interest Rates:", override_interest_rates)

    extra_data_for_optimization = await get_extra_data_for_optimization(total_assets_under_management, mpc_wallet, current_allocations, configs, override_interest_rates)

    optimized_allocations = optimize_chain_allocation_with_direction(data=extra_data_for_optimization)

    print("Optimized Allocations:", optimized_allocations)
    
    near_local_signer = KeyPair.from_string(one_time_signer_private_key)
    near_wallet = NearWallet(
        keypair=near_local_signer,
        account_id=one_time_signer_account_id,
        provider_factory=near_factory_provider,
        supported_networks=[Network.NEAR_TESTNET, Network.NEAR_MAINNET],
    )

    rebalance_operations = compute_rebalance_operations(current_allocations, optimized_allocations["allocations"])
    print("Rebalance Operations:", rebalance_operations)

    if not rebalance_operations:
        print("No rebalance operations needed.")
        return
    
    agent_public_key = Kdf.derive_public_key(
        root_public_key_str=Kdf.get_root_public_key("testnet"), # TODO: FIX
        epsilon=Kdf.derive_epsilon(account_id=contract_id, path=PATH)
    )
    agent_address = get_evm_address(agent_public_key)
    print(f"Agent Address: {agent_address}")

    await execute_all_rebalances(
        rebalance_operations=rebalance_operations,
        near_client=near_client,
        evm_factory_provider=alchemy_factory_provider,
        near_wallet=near_wallet,
        near_contract_id=contract_id,
        agent_address=agent_address,
        max_bridge_fee=to_usdc_units(max_bridge_fee),
        min_finality_threshold=min_bridge_finality_threshold,
    )
    print("Rebalance operations computed successfully.")


if __name__ == "__main__":
    asyncio.run(main())

