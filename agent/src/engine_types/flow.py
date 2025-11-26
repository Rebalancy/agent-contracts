from enum import Enum, auto


class Flow(Enum):
    RebalancerToAave = auto()
    AaveToRebalancer = auto()
    AaveToAave       = auto()