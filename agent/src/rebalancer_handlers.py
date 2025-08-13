from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider
from near_omni_client.networks.network import Network

PAYLOAD_TYPE_MAP = {
    0: "AaveSupply",
    1: "AaveWithdraw",
    2: "CCTPBurn",
    3: "CCTPMint",
    4: "RebalancerHarvest",
    5: "RebalancerInvest",
}

def get_handler_by_tx_type(tx_type: int):
    action_name = PAYLOAD_TYPE_MAP.get(tx_type, f"Unknown({tx_type})")
    handler = ACTION_MAP.get(action_name)
    
    if handler is None:
        print(f"⚠️ No hay handler para {action_name}")
        return None
    
    return handler

def handle_aave_supply(tx: bytes, evm_factory_provider:AlchemyFactoryProvider):
    print("🔵 [AaveSupply]")
    # return send_to_chain(tx, chain="ethereum")

def handle_aave_withdraw(tx: bytes, evm_factory_provider:AlchemyFactoryProvider):
    print("🔵 [AaveWithdraw]")
    # return send_to_chain(tx, chain="ethereum")

def handle_cctp_burn(tx: bytes, evm_factory_provider:AlchemyFactoryProvider):
    print("🔥 [CCTPBurn]")
    return send_to_chain(tx, network=Network.ARBITRUM_SEPOLIA, evm_factory_provider=evm_factory_provider)

def handle_cctp_mint(tx: bytes, evm_factory_provider:AlchemyFactoryProvider):
    print("🔥 [CCTPMint]")
    return send_to_chain(tx, network=Network.OPTIMISM_SEPOLIA, evm_factory_provider=evm_factory_provider)

# TODO: Creo que no la voy a necesitar....
def handle_rebalancer_harvest(tx: bytes, evm_factory_provider:AlchemyFactoryProvider):
    print("🌾 [RebalancerHarvest]")
    # return send_to_chain(tx, chain="ethereum")

def handle_rebalancer_invest(tx: bytes, evm_factory_provider:AlchemyFactoryProvider):
    print("💰 [RebalancerInvest]")
    return send_to_chain(tx, network=Network.ARBITRUM_SEPOLIA, evm_factory_provider=evm_factory_provider)

def send_to_chain(tx: bytes, network: Network, evm_factory_provider:AlchemyFactoryProvider):
    try:
        w3 = evm_factory_provider.get_provider(network)
        tx_hash = w3.eth.send_raw_transaction(tx)
        print(f"✅ Enviada a {network.name}: {tx_hash.hex()}")
        return tx_hash.hex()
    except Exception as e:
        print(f"❌ Error al enviar a {network.name}: {e}")
        return None
    
ACTION_MAP = {
    "AaveSupply": handle_aave_supply,
    "AaveWithdraw": handle_aave_withdraw,
    "CCTPBurn": handle_cctp_burn,
    "CCTPMint": handle_cctp_mint,
    "RebalancerHarvest": handle_rebalancer_harvest,
    "RebalancerInvest": handle_rebalancer_invest,
}