from typing import Protocol

class Strategy(Protocol):
    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None: ...