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