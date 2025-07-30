import os
import asyncio
from dotenv import load_dotenv
from utils import parse_chain_config,parse_chain_configs, parse_u32_result
from yield_tracker import analyze_market_data
from rebalancer import execute_rebalance
from config import get_rebalancer_config
from near_omni_client.json_rpc.client import NearClient
from near_omni_client.providers.near import NearFactoryProvider
from near_omni_client.networks import Network
from near_omni_client.providers.evm import AlchemyFactoryProvider
from near_omni_client.wallets import MPCWallet

BASE_CHAIN_ID_SEPOLIA = 84532 # TODO: Get the value from the Near Omni Client
ETHEREUM_CHAIN_ID_SEPOLIA = 111155111 # TODO: Get the value from the Near Omni Client
PATH = "rebalancer.testnet"

async def main():
    load_dotenv() 
    contract_id = os.getenv("NEAR_CONTRACT_ACCOUNT", "rebalancer.testnet")
    near_network = os.getenv("NEAR_NETWORK", "testnet")
    alchemy_api_key = os.getenv("ALCHEMY_API_KEY", "your_alchemy_api_key_here")
    asset_address = os.getenv("ASSET_ADDRESS", "0x036CbD53842c5426634e7929541eC2318f3dCF7e")
    print("asset_address:", asset_address)
    near_network = Network.parse(near_network)
    near_client = NearFactoryProvider().get_provider(near_network)
    alchemy_factory_provider = AlchemyFactoryProvider(api_key=alchemy_api_key)
    mpc_wallet = MPCWallet(
        path=PATH,
        account_id=contract_id,
        near_network=near_network,
        provider_factory=alchemy_factory_provider,
        supported_networks=[Network.BASE_SEPOLIA, Network.ETHEREUM_SEPOLIA],
    )

    chain_config_raw = await near_client.call_contract(
        contract_id=contract_id,
        method="get_all_configs",
        args={}
    )
    configs = parse_chain_configs(chain_config_raw)
    
    print("Parsed Chain Configs:", configs)

    source_chain_raw = await near_client.call_contract(
        contract_id=contract_id,
        method="get_source_chain",
        args={
            "destination_chain": BASE_CHAIN_ID_SEPOLIA
        }
    )
    source_chain = parse_u32_result(source_chain_raw)

    print("Source Chain ID:", source_chain)

    market_data = await analyze_market_data(near_client=near_client, mpc_wallet=mpc_wallet, contract_id=contract_id, source_chain_id=source_chain,
                                            asset_address=asset_address)
    # if market_data:
        # execute_rebalance()

if __name__ == "__main__":
    asyncio.run(main())
