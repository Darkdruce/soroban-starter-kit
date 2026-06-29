#![no_std]
#![deny(missing_docs)]
//! DAO governance contract template.
//!
//! Token holders create proposals and cast token-weighted votes; a proposal
//! passes when it reaches quorum with more yes votes than no votes.

use soroban_sdk::{Address, Env, String, contract, contractimpl, token};

mod errors;
mod events;
mod storage;

pub use errors::DaoError;
pub use storage::{DataKey, Proposal, ProposalKey, ProposalState, VoteKey};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn bump_persistent<K>(env: &Env, key: &K)
where
    K: soroban_sdk::TryIntoVal<Env, soroban_sdk::Val> + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// DAO governance contract for on-chain proposal creation and token-weighted voting.
///
/// Voting power is the voter's token balance at vote time (simplified snapshot).
/// A proposal passes when `yes_votes > no_votes` and total votes reach the quorum.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct DaoContract;

    #[contractimpl]
    impl DaoContract {
        /// Initialize the DAO.
        ///
        /// - `voting_period` — number of ledgers a proposal stays open for voting.
        /// - `quorum` — minimum total votes (in token units) required for a valid result.
        ///
        /// # Errors
        ///
        /// Returns [`DaoError::AlreadyInitialized`] if called again.
        pub fn initialize(
            env: Env,
            admin: Address,
            token: Address,
            voting_period: u32,
            quorum: i128,
        ) -> Result<(), DaoError> {
            if env.storage().instance().has(&DataKey::Initialized) {
                return Err(DaoError::AlreadyInitialized);
            }

            admin.require_auth();

            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage().instance().set(&DataKey::Token, &token);
            env.storage()
                .instance()
                .set(&DataKey::VotingPeriod, &voting_period);
            env.storage().instance().set(&DataKey::Quorum, &quorum);
            env.storage().instance().set(&DataKey::ProposalCount, &0u32);
            env.storage().instance().set(&DataKey::Initialized, &true);

            bump_instance(&env);
            events::initialized(&env, &admin, &token, quorum);

            Ok(())
        }

        /// Create a new proposal. The proposer must hold > 0 governance tokens.
        ///
        /// Returns the newly created `proposal_id`.
        ///
        /// # Errors
        ///
        /// Returns [`DaoError::NotInitialized`] if the DAO has not been set up.
        /// Returns [`DaoError::InsufficientVotingPower`] if the proposer has no tokens.
        pub fn create_proposal(
            env: Env,
            proposer: Address,
            title: String,
            description: String,
        ) -> Result<u32, DaoError> {
            Self::require_initialized(&env)?;
            proposer.require_auth();

            let token: Address = env
                .storage()
                .instance()
                .get(&DataKey::Token)
                .ok_or(DaoError::NotInitialized)?;
            let balance = token::Client::new(&env, &token).balance(&proposer);
            if balance <= 0 {
                return Err(DaoError::InsufficientVotingPower);
            }

            let count: u32 = env
                .storage()
                .instance()
                .get(&DataKey::ProposalCount)
                .unwrap_or(0);
            let proposal_id = count;

            let voting_period: u32 = env
                .storage()
                .instance()
                .get(&DataKey::VotingPeriod)
                .ok_or(DaoError::NotInitialized)?;
            let deadline = env.ledger().sequence() + voting_period;

            let proposal = Proposal {
                id: proposal_id,
                proposer: proposer.clone(),
                title,
                description,
                deadline,
                yes_votes: 0,
                no_votes: 0,
                state: ProposalState::Active,
            };

            env.storage()
                .persistent()
                .set(&ProposalKey::Proposal(proposal_id), &proposal);
            env.storage()
                .instance()
                .set(&DataKey::ProposalCount, &(count + 1));

            bump_instance(&env);
            bump_persistent(&env, &ProposalKey::Proposal(proposal_id));
            events::proposal_created(&env, &proposer, proposal_id);

            Ok(proposal_id)
        }

        /// Cast a vote on an active proposal. Voting weight is the voter's current token balance.
        ///
        /// # Errors
        ///
        /// Returns [`DaoError::ProposalNotFound`] if the proposal does not exist.
        /// Returns [`DaoError::InvalidState`] if the proposal is not `Active`.
        /// Returns [`DaoError::DeadlineNotReached`] if the voting period has expired (deadline passed).
        /// Returns [`DaoError::AlreadyVoted`] if the voter has already voted.
        /// Returns [`DaoError::InsufficientVotingPower`] if the voter has no tokens.
        pub fn vote(
            env: Env,
            voter: Address,
            proposal_id: u32,
            support: bool,
        ) -> Result<(), DaoError> {
            Self::require_initialized(&env)?;
            voter.require_auth();

            let mut proposal: Proposal = env
                .storage()
                .persistent()
                .get(&ProposalKey::Proposal(proposal_id))
                .ok_or(DaoError::ProposalNotFound)?;

            if proposal.state != ProposalState::Active {
                return Err(DaoError::InvalidState);
            }
            if env.ledger().sequence() > proposal.deadline {
                return Err(DaoError::DeadlineNotReached);
            }

            let vote_key = VoteKey {
                proposal_id,
                voter: voter.clone(),
            };
            if env.storage().persistent().has(&vote_key) {
                return Err(DaoError::AlreadyVoted);
            }

            let token: Address = env
                .storage()
                .instance()
                .get(&DataKey::Token)
                .ok_or(DaoError::NotInitialized)?;
            let weight = token::Client::new(&env, &token).balance(&voter);
            if weight <= 0 {
                return Err(DaoError::InsufficientVotingPower);
            }

            if support {
                proposal.yes_votes += weight;
            } else {
                proposal.no_votes += weight;
            }

            env.storage()
                .persistent()
                .set(&ProposalKey::Proposal(proposal_id), &proposal);
            env.storage().persistent().set(&vote_key, &weight);

            bump_persistent(&env, &ProposalKey::Proposal(proposal_id));
            bump_persistent(&env, &vote_key);
            events::voted(&env, &voter, proposal_id, support, weight);

            Ok(())
        }

        /// Execute a passed proposal. Callable after the deadline when quorum and majority are met.
        ///
        /// # Errors
        ///
        /// Returns [`DaoError::ProposalNotFound`] if the proposal does not exist.
        /// Returns [`DaoError::InvalidState`] if the proposal is not `Active`.
        /// Returns [`DaoError::DeadlineNotReached`] if the voting deadline has not passed.
        /// Returns [`DaoError::QuorumNotMet`] if total votes are below the quorum threshold.
        /// Returns [`DaoError::ProposalRejected`] if `no_votes >= yes_votes`.
        pub fn execute_proposal(env: Env, proposal_id: u32) -> Result<(), DaoError> {
            Self::require_initialized(&env)?;

            let mut proposal: Proposal = env
                .storage()
                .persistent()
                .get(&ProposalKey::Proposal(proposal_id))
                .ok_or(DaoError::ProposalNotFound)?;

            if proposal.state != ProposalState::Active {
                return Err(DaoError::InvalidState);
            }
            if env.ledger().sequence() <= proposal.deadline {
                return Err(DaoError::DeadlineNotReached);
            }

            let quorum: i128 = env
                .storage()
                .instance()
                .get(&DataKey::Quorum)
                .ok_or(DaoError::NotInitialized)?;
            let total_votes = proposal.yes_votes + proposal.no_votes;

            if total_votes < quorum {
                return Err(DaoError::QuorumNotMet);
            }
            if proposal.yes_votes <= proposal.no_votes {
                return Err(DaoError::ProposalRejected);
            }

            proposal.state = ProposalState::Executed;
            env.storage()
                .persistent()
                .set(&ProposalKey::Proposal(proposal_id), &proposal);

            bump_persistent(&env, &ProposalKey::Proposal(proposal_id));
            events::proposal_executed(&env, proposal_id);

            Ok(())
        }

        /// Cancel a proposal. Admin only; works only on `Active` proposals.
        ///
        /// # Errors
        ///
        /// Returns [`DaoError::NotAuthorized`] if the caller is not the admin.
        /// Returns [`DaoError::ProposalNotFound`] if the proposal does not exist.
        /// Returns [`DaoError::InvalidState`] if the proposal is not `Active`.
        pub fn cancel_proposal(env: Env, proposal_id: u32) -> Result<(), DaoError> {
            Self::require_initialized(&env)?;

            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(DaoError::NotInitialized)?;
            admin.require_auth();

            let mut proposal: Proposal = env
                .storage()
                .persistent()
                .get(&ProposalKey::Proposal(proposal_id))
                .ok_or(DaoError::ProposalNotFound)?;

            if proposal.state != ProposalState::Active {
                return Err(DaoError::InvalidState);
            }

            proposal.state = ProposalState::Cancelled;
            env.storage()
                .persistent()
                .set(&ProposalKey::Proposal(proposal_id), &proposal);

            bump_persistent(&env, &ProposalKey::Proposal(proposal_id));
            events::proposal_cancelled(&env, &admin, proposal_id);

            Ok(())
        }

        /// Return a proposal by ID.
        #[must_use]
        pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, DaoError> {
            env.storage()
                .persistent()
                .get(&ProposalKey::Proposal(proposal_id))
                .ok_or(DaoError::ProposalNotFound)
        }

        /// Return total number of proposals created.
        #[must_use]
        pub fn proposal_count(env: Env) -> u32 {
            env.storage()
                .instance()
                .get(&DataKey::ProposalCount)
                .unwrap_or(0)
        }

        fn require_initialized(env: &Env) -> Result<(), DaoError> {
            if !env.storage().instance().has(&DataKey::Initialized) {
                return Err(DaoError::NotInitialized);
            }
            Ok(())
        }
    }
}

mod test;
