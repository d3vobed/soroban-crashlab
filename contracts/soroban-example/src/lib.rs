#![no_std]
use soroban_sdk::{
    contract, contractimpl,
    map, symbol_short,
    Address, Env, Map,
};

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    /// Initialize the token with a total supply and assign it to the admin
    pub fn initialize(env: Env, admin: Address, total_supply: i128) {
        if env.storage().persistent().has(&symbol_short!("Init")) {
            panic!("already initialized");
        }

        env.storage().persistent().set(
            &symbol_short!("Admin"),
            &admin,
        );
        env.storage().persistent().set(
            &symbol_short!("Supply"),
            &total_supply,
        );

        let mut balances: Map<Address, i128> = map![&env];
        balances.set(admin.clone(), total_supply);
        env.storage().persistent().set(
            &symbol_short!("Bal"),
            &balances,
        );

        env.storage().persistent().set(
            &symbol_short!("Init"),
            &true,
        );
    }

    /// Get the total supply of tokens
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("Supply"))
            .unwrap()
    }

    /// Get the balance of an account
    pub fn balance(env: Env, account: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Bal"))
            .unwrap();
        balances.get(account).unwrap_or(0)
    }

    /// Transfer tokens from one account to another
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Bal"))
            .unwrap();

        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            panic!("insufficient balance");
        }

        balances.set(from.clone(), from_balance - amount);
        let to_balance = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, to_balance + amount);

        env.storage().persistent().set(
            &symbol_short!("Bal"),
            &balances,
        );
    }

    /// Mint new tokens (only admin)
    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("Admin"))
            .unwrap();
        
        if admin != stored_admin {
            panic!("unauthorized");
        }

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut total_supply: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("Supply"))
            .unwrap();
        total_supply += amount;
        env.storage().persistent().set(
            &symbol_short!("Supply"),
            &total_supply,
        );

        let mut balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Bal"))
            .unwrap();
        let to_balance = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, to_balance + amount);
        env.storage().persistent().set(
            &symbol_short!("Bal"),
            &balances,
        );
    }

    /// Burn tokens (only admin)
    pub fn burn(env: Env, admin: Address, from: Address, amount: i128) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("Admin"))
            .unwrap();
        
        if admin != stored_admin {
            panic!("unauthorized");
        }

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Bal"))
            .unwrap();
        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            panic!("insufficient balance");
        }
        balances.set(from, from_balance - amount);
        env.storage().persistent().set(
            &symbol_short!("Bal"),
            &balances,
        );

        let mut total_supply: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("Supply"))
            .unwrap();
        total_supply -= amount;
        env.storage().persistent().set(
            &symbol_short!("Supply"),
            &total_supply,
        );
    }

    /// Approve an allowance for a spender
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();

        if amount < 0 {
            panic!("amount cannot be negative");
        }

        let mut allowances: Map<(Address, Address), i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Allow"))
            .unwrap_or(map![&env]);
        allowances.set((owner.clone(), spender), amount);
        env.storage().persistent().set(
            &symbol_short!("Allow"),
            &allowances,
        );
    }

    /// Get the allowance for a spender
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        let allowances: Map<(Address, Address), i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Allow"))
            .unwrap_or(map![&env]);
        allowances.get((owner, spender)).unwrap_or(0)
    }

    /// Transfer tokens using allowance
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut allowances: Map<(Address, Address), i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Allow"))
            .unwrap_or(map![&env]);
        let current_allowance = allowances.get((from.clone(), spender.clone())).unwrap_or(0);
        if current_allowance < amount {
            panic!("insufficient allowance");
        }
        allowances.set((from.clone(), spender), current_allowance - amount);
        env.storage().persistent().set(
            &symbol_short!("Allow"),
            &allowances,
        );

        let mut balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&symbol_short!("Bal"))
            .unwrap();
        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            panic!("insufficient balance");
        }
        balances.set(from.clone(), from_balance - amount);
        let to_balance = balances.get(to.clone()).unwrap_or(0);
        balances.set(to, to_balance + amount);
        env.storage().persistent().set(
            &symbol_short!("Bal"),
            &balances,
        );
    }
}

#[cfg(test)]
mod test;
