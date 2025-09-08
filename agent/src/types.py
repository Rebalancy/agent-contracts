from enum import IntEnum, Enum, auto

class TxType(IntEnum):
    AaveWithdraw                  = 1
    CCTPBurn                      = 2
    CCTPMint                      = 3
    AaveSupply                    = 4
    RebalancerWithdrawToAllocate = 5
    RebalancerDeposit            = 6

class Flow(Enum):
    RebalancerToAave = auto()
    AaveToRebalancer = auto()
    AaveToAave       = auto()

def infer_flow(from_chain_id: int, to_chain_id: int, source_chain_id: int) -> Flow:
    if from_chain_id == source_chain_id:
        return Flow.RebalancerToAave
    if to_chain_id == source_chain_id:
        return Flow.AaveToRebalancer
    return Flow.AaveToAave
