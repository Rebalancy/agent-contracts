import asyncio
from dstack_sdk import DstackClient

from utils import from_chain_id_to_network
from config import Config
from helpers import Assert, BalanceHelper
from optimizer import get_extra_data_for_optimization, optimize_chain_allocation_with_direction
from engine import build_context, StrategyManager, execute_all_rebalance_operations,compute_rebalance_operations, get_allocations
from adapters import Vault, USDC
from engine import EngineContext

async def main():
    # Load configuration from environment variables
    config = Config.from_env()

    # Build engine context
    context = await build_context(config)
    print("Remote configs for all chains:", context.remote_configs)

    agent_evm_address = context.agent_address
    vault_address = context.vault_address

    # Configure Balance Helper
    BalanceHelper.configure(rebalancer_vault_address=vault_address, agent_address=agent_evm_address)

    # Configure Assert
    Assert.configure(rebalancer_vault_address=vault_address, agent_address=agent_evm_address)

    # Configure Strategies
    StrategyManager.configure(rebalancer_contract=context.rebalancer_contract, evm_factory_provider = context.evm_factory_provider, vault_address=vault_address, config=config, remote_config=context.remote_configs, agent_address=agent_evm_address)

    # Check allowances and execute transactions if required
    max_allowance = Vault(context.vault_address, context.source_network, context.evm_factory_provider).get_max_total_deposits()
    print(f"Max allowance for vault {vault_address} on source chain: {max_allowance}")
    await check_and_execute_allowances(context, max_allowance)

    current_allocations, total_assets_under_management = await get_allocations(context)
    
    extra_data_for_optimization = await get_extra_data_for_optimization(
        total_assets_under_management=total_assets_under_management,
        mpc_wallet=context.mpc_wallet,
        current_allocations=current_allocations,
        configs=context.remote_configs,
        override_interest_rates=config.override_interest_rates,
    )

    optimized_allocations = optimize_chain_allocation_with_direction(data=extra_data_for_optimization)
    print("Optimized Allocations:", optimized_allocations)
    
    rebalance_operations = compute_rebalance_operations(current_allocations, optimized_allocations["allocations"])
    print("Rebalance Operations:", rebalance_operations)

    if not rebalance_operations:
        print("No rebalance operations needed.")
        return

    # Execute Rebalance Operations 
    await execute_all_rebalance_operations(
        source_chain_id=context.source_chain_id,
        rebalance_operations=rebalance_operations
    )
    print("✅ Rebalance operations computed successfully.")


if __name__ == "__main__":
    asyncio.run(main())


async def check_and_execute_allowances(context: EngineContext, max_allowance: int):
    supported_chains = await context.rebalancer_contract.get_supported_chains()
    
    for chain_id in supported_chains:
        network_id = from_chain_id_to_network(chain_id)
        web3_instance = context.evm_factory_provider.get_provider(network_id)
        aave_lending_pool_address = context.remote_configs[chain_id]["aave"]["lending_pool_address"]
        usdc_token_address: str = context.remote_configs[chain_id]["aave"]["asset"]
        messenger_address = context.remote_config[chain_id]["cctp"]["messenger_address"]
        messenger_allowance = USDC.get_allowance(web3_instance=web3_instance, usdc_address=usdc_token_address, spender=messenger_address)

        if messenger_allowance < max_allowance:
            # we execute tx
            pass

        if chain_id == context.source_chain_id:
            # TODO: revisar si necesito la address del owner
            vault_allowance = USDC.get_allowance(web3_instance=web3_instance, usdc_address=usdc_token_address, spender=context.vault_address)
            if vault_allowance < max_allowance:
                # here we execute the allowance
                pass     
            pass 
        else:
            lending_pool_allowance = USDC.get_allowance(web3_instance=web3_instance, usdc_address=usdc_token_address, spender=aave_lending_pool_address)
            if lending_pool_allowance < max_allowance:
                # here we execute the allowance
                pass
            pass
           