// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Storage
/// @notice A simple key-value store that persists a single uint64 on-chain.
contract Storage {
    uint64 value;

    /// @notice Stores a new value on-chain.
    /// @param v The uint64 value to store.
    function set(uint64 v) public {
        value = v;
    }

    /// @notice Retrieves the currently stored value.
    /// @return The stored uint64 value.
    function get() public view returns (uint64) {
        return value;
    }
}
