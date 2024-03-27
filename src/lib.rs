use std::collections::HashMap;

// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::log_str;
use near_sdk::{env, near_bindgen, Promise};
use near_sdk::serde::{Serialize, Deserialize};

// Define the contract structure
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Project {
    pub owner: near_sdk::AccountId,
    pub name: String,
    pub description: String,
    pub target: u128,
    pub deadline: u64,
    pub amount: u128,
    pub donations:  HashMap<near_sdk::AccountId, u128>
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Crowdfunding {
    pub projects: HashMap<u64, Project>,
}

#[near_bindgen]
impl Default for Crowdfunding {
    fn default() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Crowdfunding {
    pub fn create_project(
        &mut self,
        name: String,
        description: String,
        target: u128,
        deadline: u64
    ) {
        let owner = near_sdk::env::predecessor_account_id();
        let project = Project {
            name,
            description,
            owner,
            target,
            deadline,
            amount: 0,
            donations: HashMap::new()
        };
         // Get the length of the HashMap
        let length = self.projects.len();

        // Convert the length to u64
        let length_as_u64: u64 = length as u64;
        self.projects.insert(length_as_u64 + 1, project);
    }

    pub fn get_project(&self, id: u64) -> Option<&Project> {
        self.projects.get(&id)
    }

    pub fn get_projects(&self) -> &HashMap<u64, Project> {
        &self.projects
    }

    #[payable]
    pub fn donate(&mut self, id: u64) {
        let caller_account_id = near_sdk::env::predecessor_account_id();
        let donated_amount = near_sdk::env::attached_deposit();

        if let Some(project) = self.projects.get_mut(&id) {
            let now:u64 = env::block_timestamp_ms();
            log_str(&format!("Now: {now}. Deadline {0}", project.deadline));
            if  now <= project.deadline {
                // Update the amount of the project
                project.amount += donated_amount;
    
                // Add the donation to the project's donations HashMap
                let donations = &mut project.donations;
                *donations.entry(caller_account_id.clone()).or_insert(0) += donated_amount;
            } else {
                // Handle case where deadline has passed
                near_sdk::env::panic_str("The deadline for this project has passed.");
            }
        } else {
            // Handle case where project ID does not exist
            near_sdk::env::panic_str("Project not found.");
        }
    }
    pub fn claim(&self, id: u64) {
        let owner = near_sdk::env::predecessor_account_id();
        if let Some(project) = self.projects.get(&id) {
            let now:u64 = env::block_timestamp_ms();
            log_str(&format!("Now: {now}. Deadline {0}", project.deadline));
            if  now > project.deadline {
                near_sdk::env::panic_str("The project can be claimed just after the deadline ends.");
            }
            if project.owner == owner {
                Promise::new(owner).transfer(project.amount);
            } else {
                near_sdk::env::panic_str("You are not the owner of this contract.");
            }
        } else {
            near_sdk::env::panic_str("Project not found.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env,Balance};
    use near_sdk::test_utils::VMContextBuilder;
    use test_log::test;

    const NEAR: u128 = 1000000000000000000000000;

    #[test]
    fn test_create_and_get_project() {
        let mut contract = Crowdfunding {
            projects: HashMap::new(),
        };

        // // Create project
        let deadline: u64 = env::block_timestamp();
        let name = "Test".to_string();
        let description = "Test description".to_string();
        let target = 100;
        contract.create_project(
            name.clone(),
            description.clone(),
            target.clone(),
            deadline.clone()
        );

        // Get projects
        let projects = contract.get_projects();
        let (id, project) = projects.iter().next().expect("No projects found");
        assert_eq!(projects.len(), 1);
        assert_eq!(id, &1);
        assert_eq!(project.name, name);
        assert_eq!(project.description, description);
        assert_eq!(project.target, target);
        assert_eq!(project.deadline, deadline);

        // First donation
        set_context("donation_account_2", Some(2*NEAR));
        contract.donate(*id);
        let projects = contract.get_projects();
        let (_, project) = projects.iter().next().expect("No projects found");
        let (donation_account_id, &donation) = project.donations.iter().next().expect("No donations found");
        assert_eq!(donation_account_id.to_string(), "donation_account_2");
        assert_eq!(donation, 2*NEAR);
        assert_eq!(project.amount, 2*NEAR);

        // contract.claim(*id);

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