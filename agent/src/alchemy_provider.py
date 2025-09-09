from typing import Dict, Optional
from web3 import Web3
from near_omni_client.networks.network import Network

class AlchemyProvider:
    _api_key: Optional[str] = None
    _cache: Dict[Network, Web3] = {}
    _BASE_URL_TEMPLATE = "https://{network}.g.alchemy.com/v2/{api_key}"

    @classmethod
    def configure(cls, *, api_key: str) -> None:
        if not api_key:
            raise ValueError("Alchemy api_key required")
        cls._api_key = api_key

    @classmethod
    def get_provider(cls, network: Network) -> Web3:
        if cls._api_key is None:
            raise RuntimeError("AlchemyProvider not configured. Call 'configure' with a valid api_key first.")
        if network in cls._cache:
            return cls._cache[network]
        url = cls._BASE_URL_TEMPLATE.format(network=network.value, api_key=cls._api_key)
        w3 = Web3(Web3.HTTPProvider(url))
        cls._cache[network] = w3
        return w3