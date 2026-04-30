//! Soroban SDK implementation of the Storage contract.
//!
//! Mirrors the behaviour of `storage.sol`: stores and retrieves a single
//! `u64` value in contract instance storage.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env};

/// Storage key used for the persisted value.
const KEY: soroban_sdk::Symbol = symbol_short!("VALUE");

#[contract]
pub struct StorageContract;

#[contractimpl]
impl StorageContract {
    /// Persists `v` in the contract's instance storage.
    ///
    /// Equivalent to `Storage.set(uint64 v)` in Solidity.
    pub fn set(env: Env, v: u64) {
        env.storage().instance().set(&KEY, &v);
    }

    /// Retrieves the stored value, returning `0` if nothing has been set yet.
    ///
    /// Equivalent to `Storage.get()` in Solidity.
    pub fn get(env: Env) -> u64 {
        env.storage().instance().get(&KEY).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_set_and_get() {
        let env = Env::default();
        let contract_id = env.register(StorageContract, ());
        let client = StorageContractClient::new(&env, &contract_id);

        assert_eq!(client.get(), 0);
        client.set(&42u64);
        assert_eq!(client.get(), 42);
    }
}
