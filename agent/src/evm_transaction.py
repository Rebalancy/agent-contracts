from typing import List, Optional
from dataclasses import dataclass

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
