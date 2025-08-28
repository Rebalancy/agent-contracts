from typing import List, Optional
from dataclasses import dataclass

from web3 import Web3
from gas_estimator import GasEstimator

@dataclass
class EVMTransaction:
    chain_id: Optional[int]
    nonce: Optional[int]
    gas_limit: int
    max_fee_per_gas: int
    max_priority_fee_per_gas: int
    to: Optional[str]
    value: int
    input: List[int]
    access_list: List[dict]

    def to_dict(self):
        return {
            "chain_id": self.chain_id,
            "nonce": self.nonce,
            "gas_limit": self.gas_limit,
            "max_fee_per_gas": self.max_fee_per_gas,
            "max_priority_fee_per_gas": self.max_priority_fee_per_gas,
            "to": self.to,
            "value": self.value,
            "input": self.input,
            "access_list": self.access_list,
        }

def get_empty_tx_for_chain(chain_id: int) -> EVMTransaction:
    return EVMTransaction(
        chain_id=chain_id,
        nonce=0,
        gas_limit=0,
        max_fee_per_gas=0,
        max_priority_fee_per_gas=0,
        to=None,
        value=0,
        input=[],
        access_list=[]
    )

def create_partial_tx(
    network: str,
    agent_address: str,
    evm_factory_provider,
    gas_estimator: GasEstimator
) -> EVMTransaction:
    web3 = evm_factory_provider.get_provider(network=network)
    if not web3:
        raise ValueError("Web3 provider is not initialized.")

    chain_id = web3.eth.chain_id
    nonce = web3.eth.get_transaction_count(Web3.to_checksum_address(agent_address), block_identifier="pending")
    fees = gas_estimator.get_eip1559_fees(network=network)
    print(f"Estimated fees for {network}: {fees}")
    tx = EVMTransaction(
        chain_id=chain_id,
        nonce=nonce,
        gas_limit=0,  # lo llenarás con estimate_gas cuando tengas to/data
        max_fee_per_gas=fees["max_fee_per_gas"],
        max_priority_fee_per_gas=fees["max_priority_fee_per_gas"],
        to=None,
        value=0,
        input=[],        
        access_list=[],  
    )
    return tx