// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Auth
/// @notice Demonstrates Soroban's requireAuth() builtin via Solang.
///         Only the designated owner may increment the counter.
contract Auth {
    address public owner;
    uint64 public counter;

    constructor(address _owner) public {
        owner = _owner;
    }

    /// @notice Increments the counter, requiring authentication from owner.
    /// @return The updated counter value.
    function increment() public returns (uint64) {
        owner.requireAuth();
        counter = counter + 1;
        return counter;
    }
}
