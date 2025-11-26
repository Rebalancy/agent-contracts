from engine import EngineContext
from adapters import Vault


async def get_allocations(context: EngineContext) -> tuple[dict[str, int], int]:
    current_allocations = await context.rebalancer_contract.get_allocations()

    def no_allocations_found():
        return all(v == 0 for v in current_allocations.values())
    
    if no_allocations_found():
        print("⚠️ No prior rebalance detected. Fetching totalAssets from source chain...")
        vault = Vault(context.vault_address, context.source_network, context.evm_factory_provider)
        total_assets_under_management = vault.get_total_assets()
        print(f"🔁 Using Total Assets Under Management from source chain ({context.source_chain_id}): {total_assets_under_management}")
        current_allocations[context.source_chain_id] = total_assets_under_management
        return current_allocations, total_assets_under_management

    # If NOT first rebalance → compute TAUM from allocations
    total_assets_under_management = sum(current_allocations.values())
    print(f"Total Assets Under Management (from existing allocations): {total_assets_under_management}")

    return current_allocations, total_assets_under_management