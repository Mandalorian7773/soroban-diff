//! Soroban SDK implementation of the Counter contract.
//!
//! Mirrors the behaviour of `counter.sol`: maintains a monotonically
//! incrementing `u64` counter in contract instance storage.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env};

/// Storage key for the counter value.
const KEY: soroban_sdk::Symbol = symbol_short!("COUNT");

#[contract]
pub struct CounterContract;

#[contractimpl]
impl CounterContract {
    /// Increments the on-chain counter by 1.
    ///
    /// Equivalent to `Counter.increment()` in Solidity.
    pub fn increment(env: Env) {
        let current: u64 = env.storage().instance().get(&KEY).unwrap_or(0);
        env.storage().instance().set(&KEY, &(current + 1));
    }

    /// Returns the current counter value, or `0` if never incremented.
    ///
    /// Equivalent to `Counter.get()` in Solidity.
    pub fn get(env: Env) -> u64 {
        env.storage().instance().get(&KEY).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_increment_and_get() {
        let env = Env::default();
        let contract_id = env.register(CounterContract, ());
        let client = CounterContractClient::new(&env, &contract_id);

        assert_eq!(client.get(), 0);
        client.increment();
        assert_eq!(client.get(), 1);
        client.increment();
        assert_eq!(client.get(), 2);
    }
}
