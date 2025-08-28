use alloy_primitives::U256;
use alloy_sol_types::{sol, SolCall};

sol! {
    function withdrawForCrossChainAllocation(uint256 _amountToWithdraw, uint256 _crossChainATokenBalance) returns (uint256);
    function updateCrossChainBalance(uint256 _crossChainATokenBalance) external;
    function returnFunds(uint256 amountReturned, uint256 newCrossChainATokenBalance) external;
}

pub fn encode_withdraw_for_crosschain_allocation(
    amount_to_withdraw: u128,
    cross_chain_a_token_balance: u128,
) -> Vec<u8> {
    withdrawForCrossChainAllocationCall {
        _amountToWithdraw: U256::from(amount_to_withdraw),
        _crossChainATokenBalance: U256::from(cross_chain_a_token_balance),
    }
    .abi_encode()
}

pub fn encode_update_crosschain_balance(amount: u128) -> Vec<u8> {
    updateCrossChainBalanceCall {
        _crossChainATokenBalance: U256::from(amount),
    }
    .abi_encode()
}

pub fn encode_return_funds(
    amount_returned: u128,
    new_cross_chain_a_token_balance: u128,
) -> Vec<u8> {
    returnFundsCall {
        amountReturned: U256::from(amount_returned),
        newCrossChainATokenBalance: U256::from(new_cross_chain_a_token_balance),
    }
    .abi_encode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_withdraw_for_crosschain_allocation() {
        let data = encode_withdraw_for_crosschain_allocation(1_000_000, 2_000_000);
        assert!(data.len() > 4);
    }

    #[test]
    fn test_encode_update_crosschain_balance() {
        let data = encode_update_crosschain_balance(2_500_000);
        assert!(data.len() > 4);
    }

    #[test]
    fn test_encode_return_funds() {
        let data = encode_return_funds(500_000, 3_000_000);
        assert!(data.len() > 4);
    }
}
