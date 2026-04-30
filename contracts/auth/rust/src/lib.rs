//! Soroban SDK implementation of the Auth contract.
//!
//! Mirrors `auth.sol`: only the registered owner may call `increment()`.
//! Demonstrates `Address::require_auth()` — the Rust SDK analogue of
//! Solang's `owner.requireAuth()` builtin.

#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

const OWNER: soroban_sdk::Symbol = symbol_short!("OWNER");
const COUNT: soroban_sdk::Symbol = symbol_short!("COUNT");

#[contract]
pub struct AuthContract;

#[contractimpl]
impl AuthContract {
    /// Initialises the contract by storing the owner address.
    ///
    /// Must be called once after deployment before `increment()`.
    pub fn init(env: Env, owner: Address) {
        env.storage().instance().set(&OWNER, &owner);
    }

    /// Increments the counter after verifying the caller is the owner.
    ///
    /// Equivalent to `Auth.increment()` in Solidity: calls `requireAuth()`
    /// then bumps the counter and returns the new value.
    pub fn increment(env: Env) -> u64 {
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        owner.require_auth();
        let count: u64 = env.storage().instance().get(&COUNT).unwrap_or(0);
        let new_count = count + 1;
        env.storage().instance().set(&COUNT, &new_count);
        new_count
    }

    /// Returns the current counter value without authentication.
    pub fn get_count(env: Env) -> u64 {
        env.storage().instance().get(&COUNT).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_auth_increment() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AuthContract, ());
        let client = AuthContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        client.init(&owner);

        assert_eq!(client.get_count(), 0);
        assert_eq!(client.increment(), 1);
        assert_eq!(client.increment(), 2);
    }
}
