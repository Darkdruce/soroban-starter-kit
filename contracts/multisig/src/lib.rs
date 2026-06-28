#![no_std]

#[cfg(test)]
extern crate std;

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod prop_test;
#[cfg(test)]
mod test;

pub use errors::MultisigError;
pub use storage::{DataKey, Transaction};

use soroban_common::{LEDGER_BUMP_AMOUNT, LEDGER_LIFETIME_THRESHOLD};

#[contract]
pub struct MultisigContract;

#[inline]
fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_LIFETIME_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

#[inline]
fn bump_transaction(env: &Env, tx_id: u64) {
    env.storage().persistent().extend_ttl(
        &DataKey::Transaction(tx_id),
        LEDGER_LIFETIME_THRESHOLD,
        LEDGER_BUMP_AMOUNT,
    );
}

#[inline]
fn contains(list: &Vec<Address>, address: &Address) -> bool {
    for item in list.iter() {
        if item == *address {
            return true;
        }
    }
    false
}

#[inline]
fn validate_unique_signers(signers: &Vec<Address>) -> Result<(), MultisigError> {
    if signers.is_empty() {
        return Err(MultisigError::InvalidSigners);
    }

    let mut seen = Vec::new(signers.env());
    for signer in signers.iter() {
        if contains(&seen, &signer) {
            return Err(MultisigError::InvalidSigners);
        }
        seen.push_back(signer);
    }
    Ok(())
}

#[inline]
fn validate_threshold(threshold: u32, signer_count: u32) -> Result<(), MultisigError> {
    if threshold == 0 || threshold > signer_count {
        return Err(MultisigError::InvalidThreshold);
    }
    Ok(())
}

#[contractimpl]
impl MultisigContract {
    /// Initialize the wallet with an initial signer set and threshold.
    pub fn initialize(
        env: Env,
        signers: Vec<Address>,
        threshold: u32,
    ) -> Result<(), MultisigError> {
        if env.storage().instance().has(&DataKey::Signers) {
            return Err(MultisigError::AlreadyInitialized);
        }

        validate_unique_signers(&signers)?;
        validate_threshold(threshold, signers.len())?;

        for signer in signers.iter() {
            signer.require_auth();
        }

        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        env.storage()
            .instance()
            .set(&DataKey::NextTransactionId, &0u64);
        env.storage().instance().set(&DataKey::Version, &1u32);
        bump_instance(&env);

        events::initialized(&env, threshold, signers.len());
        Ok(())
    }

    /// Add a signer and optionally adjust the threshold.
    pub fn add_signer(
        env: Env,
        approvals: Vec<Address>,
        signer: Address,
        new_threshold: u32,
    ) -> Result<(), MultisigError> {
        let mut signers = Self::get_required_signers(&env)?;
        Self::require_threshold_approvals(&env, &approvals)?;

        if contains(&signers, &signer) {
            return Err(MultisigError::InvalidSigners);
        }

        signers.push_back(signer.clone());
        validate_threshold(new_threshold, signers.len())?;

        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &new_threshold);
        bump_instance(&env);

        events::signer_added(&env, &signer, new_threshold);
        Ok(())
    }

    /// Remove a signer and optionally adjust the threshold.
    pub fn remove_signer(
        env: Env,
        approvals: Vec<Address>,
        signer: Address,
        new_threshold: u32,
    ) -> Result<(), MultisigError> {
        let signers = Self::get_required_signers(&env)?;
        Self::require_threshold_approvals(&env, &approvals)?;

        if !contains(&signers, &signer) {
            return Err(MultisigError::NotSigner);
        }

        let mut remaining = Vec::new(&env);
        for existing in signers.iter() {
            if existing != signer {
                remaining.push_back(existing);
            }
        }

        validate_unique_signers(&remaining)?;
        validate_threshold(new_threshold, remaining.len())?;

        env.storage().instance().set(&DataKey::Signers, &remaining);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &new_threshold);
        bump_instance(&env);

        events::signer_removed(&env, &signer, new_threshold);
        Ok(())
    }

    /// Propose a transaction. The proposer signs it automatically.
    pub fn propose_transaction(
        env: Env,
        proposer: Address,
        target: Address,
        function: Symbol,
        args: Vec<Val>,
    ) -> Result<u64, MultisigError> {
        Self::require_signer(&env, &proposer)?;
        proposer.require_auth();

        let tx_id = Self::propose_phase(&env, &proposer, target, function, args)?;
        events::transaction_proposed(&env, tx_id, &proposer);
        Ok(tx_id)
    }

    /// Sign a pending transaction.
    pub fn sign_transaction(env: Env, signer: Address, tx_id: u64) -> Result<(), MultisigError> {
        Self::require_signer(&env, &signer)?;
        signer.require_auth();

        let signature_count = Self::vote_phase(&env, &signer, tx_id)?;
        events::transaction_signed(&env, tx_id, &signer, signature_count);
        Ok(())
    }

    /// Execute a transaction once it has enough signatures.
    pub fn execute_transaction(env: Env, tx_id: u64) -> Result<Val, MultisigError> {
        Self::execute_phase(&env, tx_id)
    }

    pub fn get_signers(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_threshold(env: Env) -> Option<u32> {
        env.storage().instance().get(&DataKey::Threshold)
    }

    pub fn is_signer(env: Env, address: Address) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, Vec<Address>>(&DataKey::Signers)
            .is_some_and(|signers| contains(&signers, &address))
    }

    pub fn get_transaction(env: Env, tx_id: u64) -> Option<Transaction> {
        let transaction = env.storage().persistent().get(&DataKey::Transaction(tx_id));
        if transaction.is_some() {
            bump_transaction(&env, tx_id);
        }
        transaction
    }

    pub fn signature_count(env: Env, tx_id: u64) -> Option<u32> {
        Self::get_transaction(env, tx_id).map(|tx| tx.signatures.len())
    }

    // ── Phase helpers ────────────────────────────────────────────────────────

    /// Phase 1 — create a new proposal and record the proposer's implicit vote.
    #[inline]
    fn propose_phase(
        env: &Env,
        proposer: &Address,
        target: Address,
        function: Symbol,
        args: Vec<Val>,
    ) -> Result<u64, MultisigError> {
        let tx_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextTransactionId)
            .ok_or(MultisigError::NotInitialized)?;
        let mut signatures = Vec::new(env);
        signatures.push_back(proposer.clone());

        let transaction = Transaction {
            id: tx_id,
            proposer: proposer.clone(),
            target,
            function,
            args,
            signatures,
            executed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);
        env.storage()
            .instance()
            .set(&DataKey::NextTransactionId, &(tx_id + 1));
        bump_instance(env);
        bump_transaction(env, tx_id);
        Ok(tx_id)
    }

    /// Phase 2 — record a signer's vote on an existing proposal.
    #[inline]
    fn vote_phase(env: &Env, signer: &Address, tx_id: u64) -> Result<u32, MultisigError> {
        let mut transaction = Self::get_required_transaction(env, tx_id)?;
        if transaction.executed {
            return Err(MultisigError::AlreadyExecuted);
        }
        if contains(&transaction.signatures, signer) {
            return Err(MultisigError::AlreadySigned);
        }

        transaction.signatures.push_back(signer.clone());
        let signature_count = transaction.signatures.len();
        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);
        bump_transaction(env, tx_id);
        Ok(signature_count)
    }

    /// Phase 3 — verify threshold and execute the approved proposal.
    #[inline]
    fn execute_phase(env: &Env, tx_id: u64) -> Result<Val, MultisigError> {
        let mut transaction = Self::get_required_transaction(env, tx_id)?;
        if transaction.executed {
            return Err(MultisigError::AlreadyExecuted);
        }

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(MultisigError::NotInitialized)?;
        if transaction.signatures.len() < threshold {
            return Err(MultisigError::ThresholdNotMet);
        }

        transaction.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);
        bump_transaction(env, tx_id);
        events::transaction_executed(env, tx_id);

        let result: Val =
            env.invoke_contract(&transaction.target, &transaction.function, transaction.args);
        Ok(result)
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    #[inline]
    fn get_required_signers(env: &Env) -> Result<Vec<Address>, MultisigError> {
        env.storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(MultisigError::NotInitialized)
    }

    #[inline]
    fn get_required_transaction(env: &Env, tx_id: u64) -> Result<Transaction, MultisigError> {
        env.storage()
            .persistent()
            .get(&DataKey::Transaction(tx_id))
            .ok_or(MultisigError::TransactionNotFound)
    }

    #[inline]
    fn require_signer(env: &Env, signer: &Address) -> Result<(), MultisigError> {
        let signers = Self::get_required_signers(env)?;
        if !contains(&signers, signer) {
            return Err(MultisigError::NotSigner);
        }
        Ok(())
    }

    #[inline]
    fn require_threshold_approvals(
        env: &Env,
        approvals: &Vec<Address>,
    ) -> Result<(), MultisigError> {
        validate_unique_signers(approvals)?;

        let signers = Self::get_required_signers(env)?;
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(MultisigError::NotInitialized)?;
        if approvals.len() < threshold {
            return Err(MultisigError::InsufficientApprovals);
        }

        for approver in approvals.iter() {
            if !contains(&signers, &approver) {
                return Err(MultisigError::NotSigner);
            }
            approver.require_auth();
        }

        Ok(())
    }
}
