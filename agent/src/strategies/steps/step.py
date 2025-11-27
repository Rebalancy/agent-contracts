from abc import ABC, abstractmethod
from ..strategy_context import StrategyContext

class Step(ABC):
    NAME: str

    @abstractmethod
    async def run(self, ctx: StrategyContext) -> None:
        ...