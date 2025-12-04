from engine import EngineContext
from adapters import LendingPool # Vault
from helpers import BalanceHelper
from utils import from_chain_id_to_network

async def get_allocations(context: EngineContext) -> tuple[dict[str, int], int]:
    supported_chains = await context.rebalancer_contract.get_supported_chains()
    
    current_allocations = {}
    for chain_id in supported_chains:
        network_id = from_chain_id_to_network(chain_id)
        web3_instance = context.evm_factory_provider.get_provider(network_id)
        aave_lending_pool_address = context.remote_configs[chain_id]["aave"]["lending_pool_address"]
        print(f"Fetching allocation for chain ID {chain_id} using Aave Lending Pool at {aave_lending_pool_address}")
        usdc_token_address: str = context.remote_configs[chain_id]["aave"]["asset"]
        print(f"USDC Token Address on chain ID {chain_id}: {usdc_token_address}")
        a_token_address = LendingPool.get_atoken_address(web3_instance=web3_instance, aave_lending_pool_address=aave_lending_pool_address, asset_address=usdc_token_address)

        if not a_token_address:
            raise ValueError("Failed to get AToken address on destination chain.")
        
        if chain_id == context.source_chain_id: # we get the aToken Balance of the vault as holder
            current_allocations[chain_id] = BalanceHelper.get_atoken_vault_balance(web3_instance=web3_instance, atoken_address=a_token_address)
        else: # we get the aToken Balance of the agent as holder
            current_allocations[chain_id] = BalanceHelper.get_atoken_agent_balance(web3_instance=web3_instance, atoken_address=a_token_address)
    
    total_assets_under_management = sum(current_allocations.values())
    print(f"Total Assets Under Management (from existing allocations): {total_assets_under_management}")

    for chain_id, allocation in current_allocations.items():
        print(f"Chain ID {chain_id}: Current Allocation = {allocation}")

    return current_allocations, total_assets_under_management