import pytest
from rebalancer import compute_rebalance_operations

def test_no_rebalance_needed():
    current = {1: 100, 2: 200}
    optimized = {1: 100, 2: 200}
    expected = []
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_simple_rebalance():
    current = {1: 150, 2: 50}
    optimized = {1: 100, 2: 100}
    expected = [{"from": 1, "to": 2, "amount": 50}]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_multiple_sources_one_destination():
    current = {1: 150, 2: 100, 3: 50}
    optimized = {1: 100, 2: 50, 3: 150}
    expected = [
        {"from": 1, "to": 3, "amount": 50},
        {"from": 2, "to": 3, "amount": 50},
    ]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_one_source_multiple_destinations():
    current = {1: 300, 2: 0, 3: 0}
    optimized = {1: 100, 2: 100, 3: 100}
    expected = [
        {"from": 1, "to": 2, "amount": 100},
        {"from": 1, "to": 3, "amount": 100},
    ]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_partial_rebalance_only_possible():
    current = {1: 100, 2: 0}  # explícitamente decimos que 2 existe con 0
    optimized = {1: 0, 2: 200}
    expected = [{"from": 1, "to": 2, "amount": 100}]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_empty_inputs():
    current = {}
    optimized = {}
    expected = []
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_unmatched_keys():
    current = {1: 100, 2: 50, 3: 0}
    optimized = {1: 0, 2: 0, 3: 150}
    expected = [
        {"from": 1, "to": 3, "amount": 100},
        {"from": 2, "to": 3, "amount": 50},
    ]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_zero_allocation_to_destination():
    current = {1: 100, 2: 0}
    optimized = {1: 0, 2: 100}
    expected = [{"from": 1, "to": 2, "amount": 100}]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_simple_delta_transfer():
    current = {1: 100, 2: 100}
    optimized = {1: 120, 2: 80}
    expected = [{"from": 2, "to": 1, "amount": 20}]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\\nGot:      {result}"

def test_three_sources_to_one_destination():
    current = {1: 300, 2: 200, 3: 100, 4: 0}
    optimized = {1: 100, 2: 100, 3: 100, 4: 300}
    expected = [
        {"from": 1, "to": 4, "amount": 200},
        {"from": 2, "to": 4, "amount": 100},
    ]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_one_source_to_three_destinations():
    current = {1: 600, 2: 0, 3: 0, 4: 0}
    optimized = {1: 100, 2: 200, 3: 150, 4: 150}
    expected = [
        {"from": 1, "to": 2, "amount": 200},
        {"from": 1, "to": 3, "amount": 150},
        {"from": 1, "to": 4, "amount": 150},
    ]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_multi_source_multi_destination_balanced():
    current = {1: 300, 2: 200, 3: 100, 4: 0}
    optimized = {1: 100, 2: 150, 3: 100, 4: 250}
    expected = [
        {"from": 1, "to": 4, "amount": 200},
        {"from": 2, "to": 4, "amount": 50},
    ]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_new_allocation_to_zero_balance_chains():
    current = {1: 100, 2: 200, 3: 0, 4: 0}
    optimized = {1: 50, 2: 50, 3: 100, 4: 100}
    expected = [
        {"from": 1, "to": 3, "amount": 50},
        {"from": 2, "to": 3, "amount": 100},
        {"from": 2, "to": 4, "amount": 50},
    ]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"

def test_fully_drain_chain_to_fund_others():
    current = {1: 200, 2: 100, 3: 100}
    optimized = {1: 300, 2: 100, 3: 0}
    expected = [{"from": 3, "to": 1, "amount": 100}]
    result = compute_rebalance_operations(current, optimized)
    assert result == expected, f"\nExpected: {expected}\nGot:      {result}"
