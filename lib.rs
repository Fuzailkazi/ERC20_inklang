#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Balance,
        balances: ink_storage::collections::HashMap<AccountId, Balance>,
        allowances: ink_storage::collections::HashMap<(AccountId, AccountId), Balance>,
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = ink_storage::collections::HashMap::new();
            balances.insert(caller, initial_supply);

            Self {
                total_supply: initial_supply,
                balances,
                allowances: Default::default(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            *self.balances.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), ()> {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), ()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<(), ()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(());
            }
            self.transfer_from_to(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            *self.allowances.get(&(owner, spender)).unwrap_or(&0)
        }

        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<(), ()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(());
            }
            self.balances.insert(from, from_balance - value);

            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(contract.env().caller()), 100);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc20::new(100);
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            assert_eq!(contract.balance_of(accounts.bob), 0);
            assert!(contract.transfer(accounts.bob, 10).is_ok());
            assert_eq!(contract.balance_of(accounts.bob), 10);
            assert_eq!(contract.balance_of(accounts.alice), 90);
        }

        #[ink::test]
        fn approve_works() {
            let mut contract = Erc20::new(100);
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            assert!(contract.approve(accounts.bob, 50).is_ok());
            assert_eq!(contract.allowance(accounts.alice, accounts.bob), 50);
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            assert!(contract.approve(accounts.bob, 50).is_ok());
            assert!(contract.transfer_from(accounts.alice, accounts.eve, 10).is_err());

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
            assert!(contract.transfer_from(accounts.alice, accounts.eve, 10).is_ok());
            assert_eq!(contract.balance_of(accounts.eve), 10);
            assert_eq!(contract.allowance(accounts.alice, accounts.bob), 40);
        }
    }
}
