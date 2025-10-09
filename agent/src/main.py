import asyncio

from near_omni_client.providers.near import NearFactoryProvider
from near_omni_client.networks import Network
from near_omni_client.providers.evm import AlchemyFactoryProvider
from near_omni_client.wallets import MPCWallet
from near_omni_client.wallets.near_wallet import NearWallet
from near_omni_client.crypto.keypair import KeyPair
from near_omni_client.providers.near import NearFactoryProvider
from near_omni_client.chain_signatures.kdf import Kdf
from near_omni_client.chain_signatures.utils import get_evm_address

from vault_abi import get_vault_abi
from utils import from_chain_id_to_network
from optimizer_data_fetcher import get_extra_data_for_optimization
from optimizer import optimize_chain_allocation_with_direction
from rebalancer import compute_rebalance_operations
from rebalancer_executor import execute_all_rebalance_operations
from rebalancer_contract import RebalancerContract
from strategy_manager import StrategyManager
from gas_estimator import GasEstimator
from config import Config

PATH = "ethereum-1"

async def main():
    # Load configuration from environment variables
    config = Config.from_env()

    near_factory_provider = NearFactoryProvider()
    near_client = near_factory_provider.get_provider(config.near_network)
    alchemy_factory_provider = AlchemyFactoryProvider(api_key=config.alchemy_api_key)

    agent_public_key = Kdf.derive_public_key(
        root_public_key_str=Kdf.get_root_public_key(config.network_short_name),
        epsilon=Kdf.derive_epsilon(account_id=config.contract_id, path=PATH)
    )
    agent_evm_address = get_evm_address(agent_public_key)
    print(f"Agent Address: {agent_evm_address}")

    gas_estimator = GasEstimator(evm_factory_provider=alchemy_factory_provider)
    near_local_signer = KeyPair.from_string(config.one_time_signer_private_key)
    near_wallet = NearWallet(
        keypair=near_local_signer,
        account_id=config.one_time_signer_account_id,
        provider_factory=near_factory_provider,
        supported_networks=[Network.NEAR_TESTNET, Network.NEAR_MAINNET],
    )

    rebalancer_contract = RebalancerContract(near_client, near_wallet, config.contract_id, agent_evm_address, gas_estimator=gas_estimator, evm_provider=alchemy_factory_provider, config=config)
    
    configs = await rebalancer_contract.get_all_configs()
    source_chain_id = await rebalancer_contract.get_source_chain()
    current_allocations = await rebalancer_contract.get_allocations()

    def no_allocations_found():
        return all(v == 0 for v in current_allocations.values())

    source_chain_config = configs.get(source_chain_id, None)
    if not source_chain_config:
        raise ValueError(f"Source chain config for chain ID {source_chain_id} not found in configs.")

    vault_address = source_chain_config["rebalancer"]["vault_address"]
    if not vault_address:
        raise ValueError(f"Vault address for source chain {source_chain_id} not found in configs.")

    mpc_wallet = MPCWallet(
        path=PATH,
        account_id=config.contract_id, # The account ID of the contract on NEAR
        near_network=config.near_network,
        provider_factory=alchemy_factory_provider,
        supported_networks=[Network.OPTIMISM_SEPOLIA, Network.ARBITRUM_SEPOLIA],
    )

    if no_allocations_found():
        print("⚠️ No prior rebalance detected. Fetching totalAssets from source chain...")
        web3 = mpc_wallet.get_web3(from_chain_id_to_network(source_chain_id))
        contract = web3.eth.contract(address=vault_address, abi=get_vault_abi())
        total_assets_under_management = contract.functions.totalAssets().call()
        print(f"🔁 Using Total Assets Under Management from source chain ({source_chain_id}): {total_assets_under_management}")
        current_allocations[source_chain_id] = total_assets_under_management

    print("Current Allocations after fetching totalAssets:", current_allocations)

    extra_data_for_optimization = await get_extra_data_for_optimization(total_assets_under_management, mpc_wallet, current_allocations, configs, config.override_interest_rates)

    optimized_allocations = optimize_chain_allocation_with_direction(data=extra_data_for_optimization)
    print("Optimized Allocations:", optimized_allocations)
    
    rebalance_operations = compute_rebalance_operations(current_allocations, optimized_allocations["allocations"])
    print("Rebalance Operations:", rebalance_operations)

    if not rebalance_operations:
        print("No rebalance operations needed.")
        return

    # Configure Strategies
    StrategyManager.configure(rebalancer_contract=rebalancer_contract, evm_factory_provider = alchemy_factory_provider, vault_address=vault_address, config=config, remote_config=configs)

    # Execute Rebalance Operations 
    await execute_all_rebalance_operations(
        source_chain_id=source_chain_id,
        rebalance_operations=rebalance_operations
    )
    print("Rebalance operations computed successfully.")


if __name__ == "__main__":
    asyncio.run(main())

