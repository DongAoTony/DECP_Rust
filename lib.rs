#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
// use parity-scale-codec::alloc::string::ToString;

#[ink::contract]
mod decp {
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    static mut FRAGMENT_CONTAINER_ID: u64 = 0;
    // FRAGMENT_CONTAINER_ID.fetch_add(1, Ordering::SeqCst);

    
    struct FragmentContainerOwner {
        owner: AccountId,
        containerId: u64,
        startFund: u64,
        ProfitOrLoss: i64,
    }
    struct ContainerStatus {
        containerId: u64,
        status: bool,
    }
    /*
    struct FragmentContainerEA {
        ea: AccountId,
        containerId: u64,
    }
    */

    /// Create storage for DECP contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Decp {
        /// Total token supply.
        total_supply: Balance,
        /// Token name.
        name: String,
        /// Token Decimal.
        decimal: u8,
        /// Mapping from owner to number of owned tokens.
        balances: Mapping<AccountId, Balance>,
        /// Balances that can be transferred by non-owners: (owner, spender) -> allowed
        allowances: Mapping<(AccountId, AccountId), Balance>,

        /// FragmentContainer
        /// owner and containerId of this fragmentContainer
        //containers: Mapping<(((AccountId, u64), AccountId), u64), i64>,
        containers: Mapping<u64, bool>,
        containers_of_owner: Mapping<AccountId, Vec<(u64, bool)>>,
        containersOfOwners: Mapping<(AccountId, u64), (u64, i64)>,
        containersFollowWithEAs: Mapping<(AccountId, u64), (u64, i64)>,
        //startupFund: u64,
        // EA award of this fragmentContainer
        //eaId: Option<AccountId>,
        //ProfitOrLoss: i64,
        //fragmentContainer: FragmentContainer,
    }

    

    /// Create transfer event for DECP contract.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Create approval event for DECP contract.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// Specify DECP error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Return if the balance cannot fulfill a request.
        InsufficientBalance,
        InsufficientAllowance,
        InsufficientAmount,
        DividerBeZero,
    }
    /// Specify the DECP result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Decp {
        /// Create a new DECP contract with an initial supply.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance, name: String, decimal: u8) -> Self {
            // Initialize mapping for the contract.
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, initial_supply, name, decimal)
            })
        }

        /// Initialize the DECP contract with the specified initial supply.
        fn new_init(&mut self, initial_supply: Balance, name: String, decimal: u8) {
            let caller = Self::env().caller();
            self.balances.insert(&caller, &initial_supply);
            self.name = name.clone();
            self.decimal = decimal;
            self.total_supply = initial_supply;
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });
            
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /*
        /// Returns the token's name.
        #[ink(message)]
        pub fn name(&self) -> String {
            self.name.clone()
        }

        /// Returns the token's decimal.
        #[ink(message)]
        pub fn decimal(&self) -> u8 {
            self.decimal
        }
        */

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

      
        /// Transfer DECP from caller to the to aaccount
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
         ) -> Result<()> {
             let from_balance = self.balance_of_impl(from);
             if from_balance < value {
                 return Err(Error::InsufficientBalance)
             }
         
             self.balances.insert(from, &(from_balance - value));
             let to_balance = self.balance_of_impl(to);
             self.balances.insert(to, &(to_balance + value));
             self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
             });
             Ok(())
         }
         #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Approve  DECP from caller to the spender account
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }
        /// Allowance  DECP from owner & caller to the spender account
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }
        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Transfers tokens on the behalf of the `from` account to the `to account
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(&from, &to, value)?;
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }
                
        /// Fragment the asset according to user's order
        #[ink(message)]
        pub fn fragment(
            &mut self,
            totalAmount: u64,
            unit: u64,
        ) -> Result<()> {
            if totalAmount <= 0 {
                return Err(Error::InsufficientAmount)
            }
            if unit == 0 {
                return Err(Error::DividerBeZero)
            }
            let caller = self.env().caller();
            let mut amount = totalAmount;
            let mut div = amount / unit;
            while div > 0 {
                if div > 0  {
                    self.create_fragment_container(&caller, unit);
                    amount = amount - unit;
                    div = amount / unit;
                } 
                if div == 0 {
                    self.create_fragment_container(&caller, amount);
                }
            }
            
            
            Ok(())
        }
        #[inline]
        fn create_fragment_container(&mut self, owner: &AccountId, unit: u64) {
            let mut vector: Vec<(u64, bool)> = self.containers_of_owner.get(owner).unwrap_or_default();
            unsafe {
                FRAGMENT_CONTAINER_ID += 1;
                self.containersOfOwners.insert((&owner, FRAGMENT_CONTAINER_ID), &(unit, 0));
                vector.push((FRAGMENT_CONTAINER_ID, true));
                self.containers_of_owner.insert(&owner, &vector);
                self.containers.insert(FRAGMENT_CONTAINER_ID, &true);
            }
            
        }
        fn fragment_container_owner(&self, owner: AccountId, containerId: u64) -> (u64, i64)  {
            self.containersOfOwners.get((owner, containerId)).unwrap_or_default()
        }
        fn fragment_container_vector_owner(&self, owner: AccountId, containerId: u64) -> Vec<(u64, bool)>  {
            self.containers_of_owner.get(owner).unwrap_or_default()
        }
    }

        #[cfg(test)]
        mod tests {
        use super::*;
    
        use ink_lang as ink;
    
        #[ink::test]
        fn new_works() {
            let contract = Decp::new(777, "DECP".to_string(), 2);
            assert_eq!(contract.total_supply(), 777);
        }
    
        #[ink::test]
        fn balance_works() {
            let contract = Decp::new(100, "DECP".to_string(), 2);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut decp = Decp::new(100, "DECP".to_string(), 2);
            assert_eq!(decp.balance_of(AccountId::from([0x0; 32])), 0);
            assert_eq!(decp.transfer(AccountId::from([0x0; 32]), 10), Ok(()));
            assert_eq!(decp.balance_of(AccountId::from([0x0; 32])), 10);
        }

        
        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Decp::new(100, "DECP".to_string(), 2);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 20);
            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
        }
        
        #[ink::test]
        fn allowances_works() {
            let mut contract = Decp::new(100, "DECP".to_string(), 2);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 200);
            assert_eq!(contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])), 200);

            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 50);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])), 150);

            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])), 150);
        }
        
        #[ink::test]
        fn fragment_works() {
            let mut contract = Decp::new(100, "DECP".to_string(), 2);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.fragment(10, 3);
            assert_eq!(contract.fragment_container_owner(AccountId::from([0x1; 32]), 4), (1, 0));
        }
    }

}