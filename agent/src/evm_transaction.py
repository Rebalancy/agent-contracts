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

def get_empty_tx_for_chain(chain_id: int) -> EVMTransaction:
    return EVMTransaction(
        chain_id=chain_id,
        nonce=None,
        gas_limit=0,
        max_fee_per_gas=0,
        max_priority_fee_per_gas=0,
        to=None,
        value=0,
        input=[],
        access_list=[]
    )
