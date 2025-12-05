from ..strategy_context import StrategyContext
from .step import Step

class DepositIntoRebalancerAfterAssertion(Step):
    NAME = "DepositIntoRebalancerAfterAssertion"

    async def run(self, ctx: StrategyContext):
        print("Depositing into rebalancer after assertion")
        # TODO: Implement logic to deposit to rebalancer after assertion
        pass