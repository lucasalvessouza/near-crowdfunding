// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{Promise, near_bindgen};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::U128;


// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Crowdfunding {
    pub owner: near_sdk::AccountId,
    pub target: u128,
    pub deadline: u64,
    pub amount: u128,
    pub donations:  UnorderedMap<near_sdk::AccountId, u128>,
    pub account_ids: Vector<near_sdk::AccountId>,
}

// Define the default, which automatically initializes the contract
impl Default for Crowdfunding {
    fn default() -> Self {
        let owner = near_sdk::env::signer_account_id();
        Self { owner, target: 0, deadline: 0, amount: 0, donations: UnorderedMap::new(b"m"), account_ids: Vector::new(b"m") }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Crowdfunding {
    #[init]
    pub fn new(target: u128, deadline: u64) -> Self {
        let owner = near_sdk::env::signer_account_id();
        Self {
            owner,
            target,
            deadline,
            amount: 0,
            donations: UnorderedMap::new(b"m"),
            account_ids: Vector::new(near_sdk::env::current_account_id().as_bytes().to_vec()),
        }
    }

    pub fn get_owner(&self) -> near_sdk::AccountId {
        return self.owner.clone();
    }

    pub fn get_target(&self) -> u128 {
        return self.target.clone();
    }

    pub fn get_deadline(&self) -> u64 {
        return self.deadline.clone();
    }

    pub fn get_donations(&self) -> Vec<(String, U128)> {
        return self.account_ids
            .iter()
            .map(|account_id| {
                let amount = U128(self.donations.get(&account_id).unwrap_or(0));
                (account_id.to_string(), amount)
            })
            .collect();
    }

    pub fn get_amount(&self) -> U128 {
        return U128(self.amount);
    }

    #[payable]
    pub fn donate(&mut self) {
        let caller_account_id = near_sdk::env::signer_account_id();
        let donated_amount = near_sdk::env::attached_deposit();
        self.donations.insert(&caller_account_id, &donated_amount);
        self.account_ids.push(&caller_account_id);
        self.amount += donated_amount;
    }

    #[private]
    pub fn claim(&self) {
        Promise::new(self.owner.clone()).transfer(self.amount);
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env,Balance};

    const NEAR: u128 = 1000000000000000000000000;


    #[test]
    fn set_then_donate() {
        set_context("test_account", None);
        let mut contract = Crowdfunding::new(
            5*NEAR,
            1706897045
        );
        assert_eq!(
            contract.get_owner(),
            "test_account".parse().unwrap()
        );
        assert_eq!(
            contract.get_target(),
            5*NEAR
        );
        assert_eq!(
            contract.get_deadline(),
            1706897045
        );
        
        set_context("donation_account_1", Some(1*NEAR));
        contract.donate();
        assert_eq!(
            contract.get_donations(),
            vec![("donation_account_1".to_string(), U128(1*NEAR))]
        );
        assert_eq!(
            contract.get_amount(),
            U128(1*NEAR)
        );

        set_context("donation_account_2", Some(2*NEAR));
        contract.donate();
        assert_eq!(
            contract.get_donations(),
            vec![
                ("donation_account_1".to_string(), U128(1*NEAR)),
                ("donation_account_2".to_string(), U128(2*NEAR))
            ]
        );
        assert_eq!(
            contract.get_amount(),
            U128(3*NEAR)
        );

        // contract.claim();
        // assert_eq!(
        //     contract.get_amount(),
        //     U128(0)
        // );


        // Auxiliar fn: create a mock context
        fn set_context(predecessor: &str, amount: Option<Balance>) {
            let mut builder = VMContextBuilder::new();
            builder.predecessor_account_id(predecessor.parse().unwrap());
            if let Some(value) = amount {
                builder.attached_deposit(value);
            }
            testing_env!(builder.build());
        }
    }
}
