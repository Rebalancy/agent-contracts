from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider
from near_omni_client.networks.network import Network
from .abis import VAULT_ABI

class Vault:
    def __init__(self, vault_address: str, network: Network, evm_factory_provider: AlchemyFactoryProvider):
        self.address = vault_address
        self.evm_factory_provider = evm_factory_provider
        self.network = network
        web3 = self.evm_factory_provider.get_provider(network)
        self.contract = web3.eth.contract(address=vault_address, abi=VAULT_ABI)

    def get_total_assets(self) -> int:
        return self.contract.functions.totalAssets().call()