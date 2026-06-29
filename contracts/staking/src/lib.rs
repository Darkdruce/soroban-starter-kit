#![no_std]
#![deny(missing_docs)]
//! Staking rewards contract template.
//!
//! Users stake tokens to earn rewards that accrue over time from a reward pool
//! funded by the admin; rewards can be claimed and stake withdrawn at any time.

use soroban_sdk::{Address, Env, contract, contractimpl, token};

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

pub use errors::StakingError;
pub use storage::{DataKey, REWARD_SCALE};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD, extend_ttl_instance};

fn bump(env: &Env) {
    extend_ttl_instance(env, LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Returns the current global reward-per-token accumulator.
fn reward_per_token(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::RewardPerTokenStored)
        .unwrap_or(0i128)
}

/// Helper to get admin address or return NotInitialized error.
fn get_admin(env: &Env) -> Result<Address, StakingError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(StakingError::NotInitialized)
}

/// Helper to get stake token address or return NotInitialized error.
fn get_stake_token(env: &Env) -> Result<Address, StakingError> {
    env.storage()
        .instance()
        .get(&DataKey::StakeToken)
        .ok_or(StakingError::NotInitialized)
}

/// Helper to get reward token address or return NotInitialized error.
fn get_reward_token(env: &Env) -> Result<Address, StakingError> {
    env.storage()
        .instance()
        .get(&DataKey::RewardToken)
        .ok_or(StakingError::NotInitialized)
}

/// Helper to get total staked or return NotInitialized error.
fn get_total_staked_internal(env: &Env) -> Result<i128, StakingError> {
    env.storage()
        .instance()
        .get(&DataKey::TotalStaked)
        .ok_or(StakingError::NotInitialized)
}

/// Helper to get total rewards or return NotInitialized error.
fn get_total_rewards_internal(env: &Env) -> Result<i128, StakingError> {
    env.storage()
        .instance()
        .get(&DataKey::TotalRewards)
        .ok_or(StakingError::NotInitialized)
}

/// Pure reward calculation — isolated from storage for testability.
pub(crate) fn calculate_earned(stake: i128, rpt: i128, paid: i128, accrued: i128) -> i128 {
    accrued + stake * (rpt - paid) / REWARD_SCALE
}

/// Computes how many reward tokens `staker` has earned since their last update.
fn earned(env: &Env, staker: &Address) -> i128 {
    let stake: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Stake(staker.clone()))
        .unwrap_or(0i128);
    let rpt = reward_per_token(env);
    let paid: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::RewardPerTokenPaid(staker.clone()))
        .unwrap_or(0i128);
    let accrued: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Rewards(staker.clone()))
        .unwrap_or(0i128);
    calculate_earned(stake, rpt, paid, accrued)
}

/// Snapshots the staker's earned rewards and updates their paid-up-to pointer.
fn update_reward(env: &Env, staker: &Address) {
    let e = earned(env, staker);
    let rpt = reward_per_token(env);
    env.storage()
        .persistent()
        .set(&DataKey::Rewards(staker.clone()), &e);
    env.storage()
        .persistent()
        .set(&DataKey::RewardPerTokenPaid(staker.clone()), &rpt);
}

/// Simple proportional token staking contract.
///
/// Flow:
/// 1. Admin calls `initialize` — sets the stake and reward token addresses.
/// 2. Admin calls `add_rewards` to deposit reward tokens into the pool.
///    The global reward-per-token accumulator is updated proportionally.
/// 3. Users call `stake` to deposit stake tokens.
/// 4. Users call `claim_rewards` to collect accrued rewards.
/// 5. Users call `unstake` to withdraw their stake tokens.
pub use contract::*;

// The `#[contract]` / `#[contractimpl]` macros generate an undocumented public
// client type. Confine the missing_docs allowance to this module and re-export
// the public contract API above, keeping the rest of the crate enforced.
mod contract {
    #![allow(missing_docs)]
    use super::*;

    #[contract]
    pub struct StakingContract;

    #[contractimpl]
    impl StakingContract {
        /// Initialize the staking contract.
        ///
        /// # Errors
        /// - [`StakingError::AlreadyInitialized`] if called more than once.
        pub fn initialize(
            env: Env,
            admin: Address,
            stake_token: Address,
            reward_token: Address,
        ) -> Result<(), StakingError> {
            if env.storage().instance().has(&DataKey::Admin) {
                return Err(StakingError::AlreadyInitialized);
            }
            admin.require_auth();

            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage()
                .instance()
                .set(&DataKey::StakeToken, &stake_token);
            env.storage()
                .instance()
                .set(&DataKey::RewardToken, &reward_token);
            env.storage().instance().set(&DataKey::TotalStaked, &0i128);
            env.storage().instance().set(&DataKey::TotalRewards, &0i128);
            env.storage()
                .instance()
                .set(&DataKey::RewardPerTokenStored, &0i128);

            bump(&env);
            events::initialized(&env, &admin, &stake_token, &reward_token);
            Ok(())
        }

        /// Deposit `amount` stake tokens from `staker` into the contract.
        ///
        /// # Errors
        /// - [`StakingError::NotInitialized`] if the contract has not been initialized.
        /// - [`StakingError::InvalidAmount`] if `amount` <= 0.
        pub fn stake(env: Env, staker: Address, amount: i128) -> Result<(), StakingError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(StakingError::NotInitialized);
            }
            if amount <= 0 {
                return Err(StakingError::InvalidAmount);
            }
            staker.require_auth();

            update_reward(&env, &staker);

            let stake_token = get_stake_token(&env)?;
            token::Client::new(&env, &stake_token).transfer(
                &staker,
                &env.current_contract_address(),
                &amount,
            );

            let prev: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Stake(staker.clone()))
                .unwrap_or(0i128);
            let new_stake = prev + amount;
            env.storage()
                .persistent()
                .set(&DataKey::Stake(staker.clone()), &new_stake);

            let total = get_total_staked_internal(&env)?;
            env.storage()
                .instance()
                .set(&DataKey::TotalStaked, &(total + amount));

            bump(&env);
            events::staked(&env, &staker, amount, new_stake);
            Ok(())
        }

        /// Withdraw `amount` stake tokens back to `staker`.
        ///
        /// Accrued rewards are snapshotted but not transferred; call `claim_rewards` separately.
        ///
        /// # Errors
        /// - [`StakingError::NotInitialized`] if the contract has not been initialized.
        /// - [`StakingError::InvalidAmount`] if `amount` <= 0.
        /// - [`StakingError::NoStake`] if the staker has no stake.
        /// - [`StakingError::InsufficientStake`] if `amount` exceeds the staker's stake.
        pub fn unstake(env: Env, staker: Address, amount: i128) -> Result<(), StakingError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(StakingError::NotInitialized);
            }
            if amount <= 0 {
                return Err(StakingError::InvalidAmount);
            }
            staker.require_auth();

            let current: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Stake(staker.clone()))
                .unwrap_or(0i128);
            if current == 0 {
                return Err(StakingError::NoStake);
            }
            if amount > current {
                return Err(StakingError::InsufficientStake);
            }

            update_reward(&env, &staker);

            let remaining = current - amount;
            env.storage()
                .persistent()
                .set(&DataKey::Stake(staker.clone()), &remaining);

            let total = get_total_staked_internal(&env)?;
            env.storage()
                .instance()
                .set(&DataKey::TotalStaked, &(total - amount));

            let stake_token = get_stake_token(&env)?;
            token::Client::new(&env, &stake_token).transfer(
                &env.current_contract_address(),
                &staker,
                &amount,
            );

            bump(&env);
            events::unstaked(&env, &staker, amount, remaining);
            Ok(())
        }

        /// Transfer all accrued reward tokens to `staker`.
        ///
        /// # Errors
        /// - [`StakingError::NotInitialized`] if the contract has not been initialized.
        /// - [`StakingError::NoRewards`] if there are no rewards to claim.
        pub fn claim_rewards(env: Env, staker: Address) -> Result<i128, StakingError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(StakingError::NotInitialized);
            }
            staker.require_auth();

            update_reward(&env, &staker);

            let reward: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Rewards(staker.clone()))
                .unwrap_or(0i128);
            if reward <= 0 {
                return Err(StakingError::NoRewards);
            }

            env.storage()
                .persistent()
                .set(&DataKey::Rewards(staker.clone()), &0i128);

            let total_rewards = get_total_rewards_internal(&env)?;
            env.storage()
                .instance()
                .set(&DataKey::TotalRewards, &(total_rewards - reward));

            let reward_token = get_reward_token(&env)?;
            token::Client::new(&env, &reward_token).transfer(
                &env.current_contract_address(),
                &staker,
                &reward,
            );

            bump(&env);
            events::rewards_claimed(&env, &staker, reward);
            Ok(reward)
        }

        /// Admin deposits `amount` reward tokens into the pool.
        ///
        /// The reward-per-token accumulator is increased by `amount / total_staked`.
        /// If no tokens are currently staked the rewards are held and distributed
        /// when stakers join.
        ///
        /// # Errors
        /// - [`StakingError::NotInitialized`] if the contract has not been initialized.
        /// - [`StakingError::Unauthorized`] if the caller is not the admin.
        /// - [`StakingError::InvalidAmount`] if `amount` <= 0.
        pub fn add_rewards(env: Env, amount: i128) -> Result<(), StakingError> {
            if !env.storage().instance().has(&DataKey::Admin) {
                return Err(StakingError::NotInitialized);
            }
            if amount <= 0 {
                return Err(StakingError::InvalidAmount);
            }

            let admin = get_admin(&env)?;
            admin.require_auth();

            let reward_token = get_reward_token(&env)?;
            token::Client::new(&env, &reward_token).transfer(
                &admin,
                &env.current_contract_address(),
                &amount,
            );

            let total_staked = get_total_staked_internal(&env)?;
            if total_staked > 0 {
                let rpt: i128 = env
                    .storage()
                    .instance()
                    .get(&DataKey::RewardPerTokenStored)
                    .unwrap_or(0i128);
                let new_rpt = rpt + amount * REWARD_SCALE / total_staked;
                env.storage()
                    .instance()
                    .set(&DataKey::RewardPerTokenStored, &new_rpt);
            }

            let total_rewards = get_total_rewards_internal(&env)?;
            let new_total = total_rewards + amount;
            env.storage()
                .instance()
                .set(&DataKey::TotalRewards, &new_total);

            bump(&env);
            events::rewards_added(&env, &admin, amount, new_total);
            Ok(())
        }

        /// Returns the staker's current stake balance.
        pub fn get_stake(env: Env, staker: Address) -> i128 {
            env.storage()
                .persistent()
                .get(&DataKey::Stake(staker))
                .unwrap_or(0i128)
        }

        /// Returns the staker's currently accrued (unclaimed) rewards.
        pub fn get_rewards(env: Env, staker: Address) -> i128 {
            if !env.storage().instance().has(&DataKey::Admin) {
                return 0;
            }
            earned(&env, &staker)
        }

        /// Returns the total amount of tokens currently staked.
        pub fn get_total_staked(env: Env) -> i128 {
            env.storage()
                .instance()
                .get(&DataKey::TotalStaked)
                .unwrap_or(0i128)
        }

        /// Returns the total reward tokens held by the contract.
        pub fn get_total_rewards(env: Env) -> i128 {
            env.storage()
                .instance()
                .get(&DataKey::TotalRewards)
                .unwrap_or(0i128)
        }
    }
}
