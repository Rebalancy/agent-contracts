use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_sol_types::{sol, SolCall, SolValue};
use std::str::FromStr;

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

pub fn compute_snapshot_digest(
    chain_id: u64,
    verifying_contract: String,
    balance: u128,
    nonce: u64,
    deadline: u64,
    assets: String,
    receiver: String,
) -> Vec<u8> {
    // struct hash
    let snapshot_typehash = B256::from(keccak256(
        b"CrossChainBalanceSnapshot(uint256 balance,uint256 nonce,uint256 deadline,uint256 assets,address receiver)",
    ));

    // convert all inputs to solidity-compatible values
    let struct_encoded = (
        snapshot_typehash,
        U256::from(balance),
        U256::from(nonce),
        U256::from(deadline),
        U256::from_str(&assets).expect("invalid assets value"),
        Address::from_str(&receiver).expect("invalid receiver address"),
    )
        .abi_encode();

    let struct_hash = B256::from(keccak256(struct_encoded));

    // domain hash
    let domain_typehash = B256::from(keccak256(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    ));
    let name_hash = B256::from(keccak256(b"AaveVault"));
    let version_hash = B256::from(keccak256(b"1"));
    let verifying_addr: Address =
        Address::from_str(&verifying_contract).expect("invalid verifying contract");

    let domain_encoded = (
        domain_typehash,
        name_hash,
        version_hash,
        U256::from(chain_id),
        verifying_addr,
    )
        .abi_encode();

    let domain_hash = B256::from(keccak256(domain_encoded));

    // final digest (EIP-712 hash)
    let mut v = Vec::with_capacity(2 + 32 + 32);
    v.extend_from_slice(&[0x19, 0x01]);
    v.extend_from_slice(domain_hash.as_slice());
    v.extend_from_slice(struct_hash.as_slice());

    keccak256(v).to_vec()
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
