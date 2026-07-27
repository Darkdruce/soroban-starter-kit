#![no_std]
#![deny(missing_docs)]
//! Ranked-choice / single-choice ballot voting contract template.
//!
//! Voters cast a single vote among registered options; results are tallied
//! on-chain once voting closes.
//!
//! # Voting window (issue #787)
//! `initialize` now accepts `voting_start` and `voting_end` ledger sequences.
//! `vote()` is rejected with [`BallotError::VotingNotStarted`] before the
//! window opens and with [`BallotError::VotingClosed`] after it closes.
//!
//! # Voter deregistration (issue #786)
//! `deregister_voter` (admin-only) removes a registered voter, but only while
//! no vote has yet been cast. Once any vote exists the function returns
//! [`BallotError::VotingAlreadyStarted`].

use soroban_sdk::{contract, contractimpl, Address, Env};

mod errors;
mod events;
mod storage;

pub use errors::BallotError;
pub use storage::DataKey;

use soroban_common::{extend_ttl_instance, LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump(env: &Env) {
    extend_ttl_instance(env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Single-choice on-chain ballot contract.
///
/// Flow:
/// 1. Admin calls `initialize` — sets up voting with a `voting_start` and
///    `voting_end` ledger window.
/// 2. Admin calls `register_voter` to add voters to the ballot. To correct a
///    mistake, admin may call `deregister_voter` before any vote is cast.
/// 3. Voters call `vote` to cast their single vote (yes=1, no=0) during the
///    voting window.
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
        /// `voting_start` and `voting_end` are inclusive ledger sequence numbers
        /// defining the window during which votes are accepted.
        ///
        /// # Errors
        /// - [`BallotError::AlreadyInitialized`] if called more than once.
        /// - [`BallotError::InvalidWindow`] if `voting_start` >= `voting_end` or
        ///   `voting_end` <= current ledger.
        pub fn initialize(
            env: Env,
            admin: Address,
            voting_start: u32,
            voting_end: u32,
        ) -> Result<(), BallotError> {
            if env.storage().instance().has(&DataKey::Admin) {
                return Err(BallotError::AlreadyInitialized);
            }
            if voting_start >= voting_end || voting_end <= env.ledger().sequence() {
                return Err(BallotError::InvalidWindow);
            }
            admin.require_auth();

            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage().instance().set(&DataKey::VotingActive, &true);
            env.storage()
                .instance()
                .set(&DataKey::VotingStart, &voting_start);
            env.storage()
                .instance()
                .set(&DataKey::VotingEnd, &voting_end);
            env.storage().instance().set(&DataKey::YesVotes, &0i128);
            env.storage().instance().set(&DataKey::NoVotes, &0i128);
            env.storage().instance().set(&DataKey::TotalVotes, &0i128);

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

            env.storage()
                .persistent()
                .set(&DataKey::RegisteredVoter(voter.clone()), &true);
            env.storage().persistent().extend_ttl(
                &DataKey::RegisteredVoter(voter.clone()),
                LEDGER_LIFETIME_THRESHOLD,
                LEDGER_BUMP_AMOUNT,
            );

            bump(&env);
            events::voter_registered(&env, &voter);
            Ok(())
        }

        /// Admin deregisters a voter, correcting a registration mistake.
        ///
        /// Only allowed before any vote has been cast. Once voting has begun
        /// the registered-voter list is frozen to maintain ballot integrity.
        ///
        /// # Errors
        /// - [`BallotError::NotInitialized`] if the contract has not been initialized.
        /// - [`BallotError::Unauthorized`] if the caller is not the admin.
        /// - [`BallotError::VotingAlreadyStarted`] if at least one vote has been cast.
        /// - [`BallotError::NotRegistered`] if the voter is not currently registered.
        pub fn deregister_voter(env: Env, voter: Address) -> Result<(), BallotError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(BallotError::NotInitialized);
            }

            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(BallotError::NotInitialized)?;
            admin.require_auth();

            // Reject once any vote has been cast.
            let total_votes: i128 = env
                .storage()
                .instance()
                .get(&DataKey::TotalVotes)
                .unwrap_or(0i128);
            if total_votes > 0 {
                return Err(BallotError::VotingAlreadyStarted);
            }

            let is_registered: bool = env
                .storage()
                .persistent()
                .get(&DataKey::RegisteredVoter(voter.clone()))
                .unwrap_or(false);
            if !is_registered {
                return Err(BallotError::NotRegistered);
            }

            env.storage()
                .persistent()
                .remove(&DataKey::RegisteredVoter(voter.clone()));

            bump(&env);
            events::voter_deregistered(&env, &voter);
            Ok(())
        }

        /// Voter casts their vote (choice: 1=yes, 0=no).
        ///
        /// The vote is only accepted within the `[voting_start, voting_end]` ledger window.
        ///
        /// # Errors
        /// - [`BallotError::NotInitialized`] if the contract has not been initialized.
        /// - [`BallotError::VotingNotStarted`] if the current ledger is before `voting_start`.
        /// - [`BallotError::VotingClosed`] if voting is inactive or past `voting_end`.
        /// - [`BallotError::NotRegistered`] if the voter is not registered.
        /// - [`BallotError::AlreadyVoted`] if the voter has already voted.
        /// - [`BallotError::InvalidChoice`] if choice is not 0 or 1.
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

            // Enforce voting window.
            let voting_start: u32 = env
                .storage()
                .instance()
                .get(&DataKey::VotingStart)
                .unwrap_or(0u32);
            let voting_end: u32 = env
                .storage()
                .instance()
                .get(&DataKey::VotingEnd)
                .unwrap_or(0u32);
            let current = env.ledger().sequence();
            if current < voting_start {
                return Err(BallotError::VotingNotStarted);
            }
            if current > voting_end {
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

            env.storage()
                .persistent()
                .set(&DataKey::Voter(voter.clone()), &true);
            env.storage().persistent().extend_ttl(
                &DataKey::Voter(voter.clone()),
                LEDGER_LIFETIME_THRESHOLD,
                LEDGER_BUMP_AMOUNT,
            );

            // Increment total-vote counter (used by deregister_voter guard).
            let total: i128 = env
                .storage()
                .instance()
                .get(&DataKey::TotalVotes)
                .unwrap_or(0i128);
            env.storage()
                .instance()
                .set(&DataKey::TotalVotes, &(total + 1));

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

mod test;
