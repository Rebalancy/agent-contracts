from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class DepositIntoRebalancer(Step):
    NAME = "DepositIntoRebalancer"

    async def run(self, ctx: StrategyContext):
        print("Depositing into rebalancer...")
        crosschain_balance = ctx.amount # TODO: Fix this with process to get the actual crosschain balance
        deposit_payload = await ctx.rebalancer_contract.build_and_sign_return_funds_tx(to_chain_id=ctx.to_chain_id, cross_chain_a_token_balance=crosschain_balance,  amount=ctx.amount, to=ctx.vault_address)

        broadcast(ctx.web3_destination, deposit_payload)

        print("Deposit transaction broadcasted successfully!")