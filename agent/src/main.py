import asyncio


from config import Config
from helpers import Assert, BalanceHelper
from optimizer import get_extra_data_for_optimization, optimize_chain_allocation_with_direction
from engine import build_context, StrategyManager, execute_all_rebalance_operations,compute_rebalance_operations, get_allocations


async def main():
    # Load configuration from environment variables
    config = Config.from_env()

    # Build engine context
    context = await build_context(config)
    print("Remote configs for all chains:", context.remote_configs)

    current_allocations, total_assets_under_management = await get_allocations(context)
    print("Current Allocations after fetching totalAssets:", current_allocations)
    
    # aclarar lo de total assets under management
    extra_data_for_optimization = await get_extra_data_for_optimization(total_assets_under_management, context.mpc_wallet, current_allocations, context.remote_configs, config.override_interest_rates)

    optimized_allocations = optimize_chain_allocation_with_direction(data=extra_data_for_optimization)
    print("Optimized Allocations:", optimized_allocations)
    
    rebalance_operations = compute_rebalance_operations(current_allocations, optimized_allocations["allocations"])
    print("Rebalance Operations:", rebalance_operations)

    if not rebalance_operations:
        print("No rebalance operations needed.")
        return

    agent_evm_address = context.agent_address
    vault_address = context.vault_address

    # Configure Balance Helper
    BalanceHelper.configure(rebalancer_vault_address=vault_address, agent_address=agent_evm_address)

    # Configure Assert
    Assert.configure(rebalancer_vault_address=vault_address, agent_address=agent_evm_address)

    # Configure Strategies
    StrategyManager.configure(rebalancer_contract=context.rebalancer_contract, evm_factory_provider = context.evm_factory_provider, vault_address=vault_address, config=config, remote_config=context.remote_configs, agent_address=agent_evm_address)

    # Execute Rebalance Operations 
    await execute_all_rebalance_operations(
        source_chain_id=context.source_chain_id,
        rebalance_operations=rebalance_operations
    )
    print("Rebalance operations computed successfully.")


if __name__ == "__main__":
    asyncio.run(main())

