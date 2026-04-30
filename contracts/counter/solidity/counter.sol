// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Counter
/// @notice A simple monotonically-incrementing counter stored on-chain.
contract Counter {
    uint64 count;

    /// @notice Increments the counter by 1.
    function increment() public {
        count += 1;
    }

    /// @notice Returns the current counter value.
    /// @return The current uint64 counter value.
    function get() public view returns (uint64) {
        return count;
    }
}
