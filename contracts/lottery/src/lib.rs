#![no_std]
#![deny(missing_docs)]
//! Commit-reveal lottery contract template.
//!
//! Players buy tickets while the lottery is open; the admin commits to a hashed
//! secret then reveals it to draw a verifiable random winner.

use soroban_sdk::{Address, Bytes, BytesN, Env, Vec, contract, contractimpl, crypto::Hash, token};

mod errors;
mod events;
mod storage;

pub use errors::LotteryError;
pub use storage::{Commit, DataKey, LotteryInfo, LotteryState};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

fn get_required<T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
) -> Result<T, LotteryError> {
    env.storage()
        .instance()
        .get(key)
        .ok_or(LotteryError::NotInitialized)
}

/// Lottery contract using a commit-reveal scheme for verifiable randomness.
///
/// Lifecycle:
/// 1. Admin calls `initialize` (→ Open).
/// 2. Anyone calls `buy_ticket` (while Open).
/// 3. Admin calls `commit` with hash(secret ++ salt) (→ Committed).
/// 4. Admin calls `draw` revealing secret and salt (→ Drawn); winner receives the prize pool.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;
    use storage::DataKey::*;
    // Explicit import so the `Commit` struct (type namespace) is unambiguous
    // alongside the glob-imported `DataKey::Commit` variant (value namespace).
    use storage::Commit;

    #[contract]
    pub struct LotteryContract;

    #[contractimpl]
    impl LotteryContract {
        /// Initialise the lottery.
        ///
        /// # Errors
        /// - [`LotteryError::AlreadyInitialized`]
        /// - [`LotteryError::InvalidTicketPrice`] if ticket_price <= 0
        pub fn initialize(
            env: Env,
            admin: Address,
            token: Address,
            ticket_price: i128,
        ) -> Result<(), LotteryError> {
            if env.storage().instance().has(&State) {
                return Err(LotteryError::AlreadyInitialized);
            }
            if ticket_price <= 0 {
                return Err(LotteryError::InvalidTicketPrice);
            }
            admin.require_auth();

            env.storage().instance().set(&Admin, &admin);
            env.storage().instance().set(&Token, &token);
            env.storage().instance().set(&TicketPrice, &ticket_price);
            env.storage()
                .instance()
                .set(&Participants, &Vec::<Address>::new(&env));
            env.storage().instance().set(&State, &LotteryState::Open);

            bump_instance(&env);
            events::initialized(&env, &admin, ticket_price);
            Ok(())
        }

        /// Purchase one ticket. Transfers `ticket_price` tokens from caller to contract.
        ///
        /// # Errors
        /// - [`LotteryError::NotInitialized`]
        /// - [`LotteryError::LotteryClosed`] if lottery is no longer Open
        pub fn buy_ticket(env: Env, buyer: Address) -> Result<(), LotteryError> {
            let state: LotteryState = get_required(&env, &State)?;
            if state != LotteryState::Open {
                return Err(LotteryError::LotteryClosed);
            }

            buyer.require_auth();

            let ticket_price: i128 = get_required(&env, &TicketPrice)?;
            let token_addr: Address = get_required(&env, &Token)?;
            token::Client::new(&env, &token_addr).transfer(
                &buyer,
                &env.current_contract_address(),
                &ticket_price,
            );

            let mut participants: Vec<Address> = get_required(&env, &Participants)?;
            participants.push_back(buyer.clone());
            env.storage().instance().set(&Participants, &participants);

            bump_instance(&env);
            events::ticket_purchased(&env, &buyer);
            Ok(())
        }

        /// Admin submits hash(secret ++ salt) to lock in randomness commitment.
        /// Transitions lottery to Committed state, closing ticket sales.
        ///
        /// # Errors
        /// - [`LotteryError::NotInitialized`]
        /// - [`LotteryError::Unauthorized`]
        /// - [`LotteryError::LotteryClosed`] if not Open
        /// - [`LotteryError::CommitAlreadySubmitted`]
        /// - [`LotteryError::NoTickets`] if no participants
        pub fn commit(env: Env, hash: BytesN<32>) -> Result<(), LotteryError> {
            let admin: Address = get_required(&env, &Admin)?;
            admin.require_auth();

            let state: LotteryState = get_required(&env, &State)?;
            if state != LotteryState::Open {
                return Err(if state == LotteryState::Committed {
                    LotteryError::CommitAlreadySubmitted
                } else {
                    LotteryError::DrawAlreadyDone
                });
            }

            let participants: Vec<Address> = get_required(&env, &Participants)?;
            if participants.is_empty() {
                return Err(LotteryError::NoTickets);
            }

            env.storage().instance().set(&Commit, &Commit { hash });
            env.storage()
                .instance()
                .set(&State, &LotteryState::Committed);

            bump_instance(&env);
            events::committed(&env, &admin);
            Ok(())
        }

        /// Admin reveals secret and salt. The contract verifies the preimage matches
        /// the stored commitment, then uses hash(secret ++ salt ++ participants_count)
        /// to pick a winner and transfer the entire prize pool.
        ///
        /// # Errors
        /// - [`LotteryError::NotInitialized`]
        /// - [`LotteryError::Unauthorized`]
        /// - [`LotteryError::DrawNotDone`] if not yet Committed
        /// - [`LotteryError::DrawAlreadyDone`] if already Drawn
        /// - [`LotteryError::RevealMismatch`] if preimage does not match commitment
        pub fn draw(
            env: Env,
            secret: BytesN<32>,
            salt: BytesN<32>,
        ) -> Result<Address, LotteryError> {
            let admin: Address = get_required(&env, &Admin)?;
            admin.require_auth();

            let state: LotteryState = get_required(&env, &State)?;
            match state {
                LotteryState::Open => return Err(LotteryError::DrawNotDone),
                LotteryState::Drawn => return Err(LotteryError::DrawAlreadyDone),
                LotteryState::Committed => {}
            }

            // Verify commitment: SHA-256(secret ++ salt) must match stored hash.
            let mut preimage = Bytes::new(&env);
            preimage.extend_from_array(&secret.to_array());
            preimage.extend_from_array(&salt.to_array());
            let computed: BytesN<32> = env.crypto().sha256(&preimage).into();

            let stored_commit: Commit = get_required(&env, &Commit)?;
            if computed != stored_commit.hash {
                return Err(LotteryError::RevealMismatch);
            }

            let participants: Vec<Address> = get_required(&env, &Participants)?;

            // Derive winner index from hash(secret ++ salt ++ participants_count).
            let count = participants.len() as u64;
            let count_bytes = count.to_be_bytes();
            let mut entropy_input = Bytes::new(&env);
            entropy_input.extend_from_array(&secret.to_array());
            entropy_input.extend_from_array(&salt.to_array());
            entropy_input.extend_from_array(&count_bytes);
            let entropy: Hash<32> = env.crypto().sha256(&entropy_input);
            let entropy_bytes = entropy.to_array();
            // Use last 8 bytes as u64 for modulo.
            let idx_raw = u64::from_be_bytes([
                entropy_bytes[24],
                entropy_bytes[25],
                entropy_bytes[26],
                entropy_bytes[27],
                entropy_bytes[28],
                entropy_bytes[29],
                entropy_bytes[30],
                entropy_bytes[31],
            ]);

            let winner_idx = (idx_raw % count) as u32;
            let winner = participants.get(winner_idx).unwrap();

            // Transfer full prize pool to winner.
            let ticket_price: i128 = get_required(&env, &TicketPrice)?;
            let prize = ticket_price * count as i128;
            let token_addr: Address = get_required(&env, &Token)?;
            token::Client::new(&env, &token_addr).transfer(
                &env.current_contract_address(),
                &winner,
                &prize,
            );

            env.storage().instance().set(&State, &LotteryState::Drawn);
            env.storage().instance().set(&Winner, &winner);

            bump_instance(&env);
            events::winner_drawn(&env, &winner, prize);
            Ok(winner)
        }

        /// Return lottery details.
        ///
        /// # Errors
        /// - [`LotteryError::NotInitialized`]
        pub fn get_info(env: Env) -> Result<LotteryInfo, LotteryError> {
            Ok(LotteryInfo {
                admin: get_required(&env, &Admin)?,
                token: get_required(&env, &Token)?,
                ticket_price: get_required(&env, &TicketPrice)?,
                state: get_required(&env, &State)?,
                participants: get_required(&env, &Participants)?,
            })
        }

        /// Return the winner address (only available after draw).
        ///
        /// # Errors
        /// - [`LotteryError::NotInitialized`]
        /// - [`LotteryError::DrawNotDone`] if draw hasn't happened yet
        pub fn get_winner(env: Env) -> Result<Address, LotteryError> {
            let state: LotteryState = get_required(&env, &State)?;
            if state != LotteryState::Drawn {
                return Err(LotteryError::DrawNotDone);
            }
            get_required(&env, &Winner)
        }
    }
}

mod test;
