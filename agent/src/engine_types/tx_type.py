from enum import IntEnum


class TxType(IntEnum):
    AaveWithdraw                  = 1
    CCTPBurn                      = 2
    CCTPMint                      = 3
    AaveSupply                    = 4
    RebalancerWithdrawToAllocate = 5
    RebalancerDeposit            = 6

