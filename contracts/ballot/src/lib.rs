#![no_std]
#![deny(missing_docs)]
//! Ranked-choice / single-choice ballot voting contract template.
//!
//! Voters cast a single vote among registered options; results are tallied
//! on-chain once voting closes.

use soroban_sdk::{contract, contractimpl, Address, Env};

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

pub use errors::BallotError;
pub use storage::DataKey;

use soroban_common::{extend_ttl_instance, LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump(env: &Env) {
    extend_ttl_instance(env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Single-choice on-chain ballot contract.
///
/// Flow:
/// 1. Admin calls `initialize` — sets up voting.
/// 2. Admin calls `register_voter` to add voters to the ballot.
/// 3. Voters call `vote` to cast their single vote (yes=1, no=0).
/// 4. Admin calls `tally` to get final results and close voting.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct BallotContract;

#[contractimpl]
impl BallotContract {
    /// Initialize the ballot contract.
    ///
    /// # Errors
    /// - [`BallotError::AlreadyInitialized`] if called more than once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), BallotError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BallotError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VotingActive, &true);
        env.storage().instance().set(&DataKey::YesVotes, &0i128);
        env.storage().instance().set(&DataKey::NoVotes, &0i128);

        bump(&env);
        events::initialized(&env, &admin);
        Ok(())
    }

    /// Admin registers a voter for the ballot.
    ///
    /// # Errors
    /// - [`BallotError::NotInitialized`] if the contract has not been initialized.
    /// - [`BallotError::Unauthorized`] if the caller is not the admin.
    pub fn register_voter(env: Env, voter: Address) -> Result<(), BallotError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(BallotError::NotInitialized);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(BallotError::NotInitialized)?;
        admin.require_auth();

        let voter_key = DataKey::RegisteredVoter(voter.clone());
        env.storage()
            .persistent()
            .set(&voter_key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&voter_key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

        bump(&env);
        events::voter_registered(&env, &voter);
        Ok(())
    }

    /// Voter casts their vote (choice: 1=yes, 0=no).
    ///
    /// # Errors
    /// - [`BallotError::NotInitialized`] if the contract has not been initialized.
    /// - [`BallotError::NotRegistered`] if the voter is not registered.
    /// - [`BallotError::AlreadyVoted`] if the voter has already voted.
    /// - [`BallotError::InvalidChoice`] if choice is not 0 or 1.
    /// - [`BallotError::VotingClosed`] if voting is not active.
    pub fn vote(env: Env, voter: Address, choice: u32) -> Result<(), BallotError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(BallotError::NotInitialized);
        }
        voter.require_auth();

        let voting_active: bool = env
            .storage()
            .instance()
            .get(&DataKey::VotingActive)
            .unwrap_or(false);
        if !voting_active {
            return Err(BallotError::VotingClosed);
        }

        let is_registered: bool = env
            .storage()
            .persistent()
            .get(&DataKey::RegisteredVoter(voter.clone()))
            .unwrap_or(false);
        if !is_registered {
            return Err(BallotError::NotRegistered);
        }

        let already_voted: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Voter(voter.clone()))
            .unwrap_or(false);
        if already_voted {
            return Err(BallotError::AlreadyVoted);
        }

        if choice != 0 && choice != 1 {
            return Err(BallotError::InvalidChoice);
        }

        let voter_key = DataKey::Voter(voter.clone());
        let registered_voter_key = DataKey::RegisteredVoter(voter.clone());
        env.storage()
            .persistent()
            .set(&voter_key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&voter_key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .extend_ttl(&registered_voter_key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);

        if choice == 1 {
            let yes: i128 = env
                .storage()
                .instance()
                .get(&DataKey::YesVotes)
                .unwrap_or(0i128);
            env.storage()
                .instance()
                .set(&DataKey::YesVotes, &(yes + 1));
        } else {
            let no: i128 = env
                .storage()
                .instance()
                .get(&DataKey::NoVotes)
                .unwrap_or(0i128);
            env.storage()
                .instance()
                .set(&DataKey::NoVotes, &(no + 1));
        }

        bump(&env);
        events::voted(&env, &voter, choice);
        Ok(())
    }

    /// Get tally results and close voting.
    ///
    /// Returns (yes_votes, no_votes).
    ///
    /// # Errors
    /// - [`BallotError::NotInitialized`] if the contract has not been initialized.
    /// - [`BallotError::Unauthorized`] if the caller is not the admin.
    pub fn tally(env: Env) -> Result<(i128, i128), BallotError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(BallotError::NotInitialized);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(BallotError::NotInitialized)?;
        admin.require_auth();

        let yes: i128 = env
            .storage()
            .instance()
            .get(&DataKey::YesVotes)
            .unwrap_or(0i128);
        let no: i128 = env
            .storage()
            .instance()
            .get(&DataKey::NoVotes)
            .unwrap_or(0i128);

        env.storage().instance().set(&DataKey::VotingActive, &false);

        bump(&env);
        events::tally_result(&env, yes, no);
        Ok((yes, no))
    }

    /// Get yes vote count.
    pub fn get_yes_votes(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::YesVotes)
            .unwrap_or(0i128)
    }

    /// Get no vote count.
    pub fn get_no_votes(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::NoVotes)
            .unwrap_or(0i128)
    }
}
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn test_ballot_lifecycle() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let voter1 = Address::random(&env);
        let voter2 = Address::random(&env);

        let contract = BallotContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin);
        contract.register_voter(&voter1);
        contract.register_voter(&voter2);

        contract.vote(&voter1, &1u32);
        contract.vote(&voter2, &0u32);

        assert_eq!(contract.get_yes_votes(), 1);
        assert_eq!(contract.get_no_votes(), 1);

        let (yes, no) = contract.tally();
        assert_eq!(yes, 1);
        assert_eq!(no, 1);
    }

    #[test]
    fn test_double_vote_prevention() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let voter = Address::random(&env);

        let contract = BallotContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin);
        contract.register_voter(&voter);

        contract.vote(&voter, &1u32);

        let result = contract.try_vote(&voter, &1u32);
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e.error(), BallotError::AlreadyVoted),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_unregistered_voter_rejected() {
        let env = Env::default();
        env.ledger().with_mut(|le| {
            le.timestamp = 1;
        });

        let admin = Address::random(&env);
        let unregistered = Address::random(&env);

        let contract = BallotContractClient::new(&env, &env.current_contract_id());

        contract.initialize(&admin);

        let result = contract.try_vote(&unregistered, &1u32);
        assert!(result.is_err());
        match result {
            Err(e) => assert_eq!(e.error(), BallotError::NotRegistered),
            _ => unreachable!(),
        }
    }
}
