import asyncio
from typing import Callable, Awaitable, TypeVar

T = TypeVar("T")

async def retry_async_step(fn: Callable[[], Awaitable[T]], retries: int = 3) -> T:
    last_exc = None
    for _ in range(retries):
        try:
            return await fn()
        except Exception as e:
            last_exc = e
            await asyncio.sleep(1)
    raise last_exc