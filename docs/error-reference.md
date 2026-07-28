# Error Reference

Comprehensive reference for all error codes returned by the contracts in this repository.

---

## Token Contract — `TokenError`

### `InsufficientBalance` (code 1)

**Description:** The caller's token balance is too low to complete the requested transfer or burn.

**Common cause:** Attempting to transfer or burn more tokens than the account currently holds.

**Resolution:** Check the account balance with `balance()` before calling `transfer` or `burn`. Ensure the amount does not exceed the available balance.

---

### `InsufficientAllowance` (code 2)

**Description:** The approved allowance is too low for the requested `transfer_from` amount.

**Common cause:** The spender is trying to move more tokens than the owner approved via `approve`.

**Resolution:** Call `allowance()` to verify the current approved amount. Ask the token owner to call `approve` with a sufficient amount before retrying.

---

### `Unauthorized` (code 3)

**Description:** The caller is not the admin or does not have permission for this operation.

**Common cause:** Calling `mint`, `set_admin`, or other admin-only functions from a non-admin address.

**Resolution:** Ensure the transaction is signed by the current admin address. Use `admin()` to confirm who holds admin rights.

---

### `AlreadyInitialized` (code 4)

**Description:** `initialize` was called on a contract that has already been set up.

**Common cause:** Calling `initialize` more than once on the same deployed contract instance.

**Resolution:** `initialize` should only be called once, immediately after deployment. Check contract state before calling it.

---

### `NotInitialized` (code 5)

**Description:** An operation was attempted before the contract was initialized.

**Common cause:** Invoking any contract function before `initialize` has been called.

**Resolution:** Call `initialize` with the required parameters (admin, name, symbol, decimals) before any other interaction.

---

### `InvalidAmount` (code 6)

**Description:** The amount is zero, negative, or exceeds the configured max supply.

**Common cause:** Passing `0` or a negative value as an amount, or minting beyond the cap set at initialization.

**Resolution:** Validate that the amount is a positive integer within the allowed range. Check the max supply with `max_supply()` if applicable.

---

### `Overflow` (code 7)

**Description:** Arithmetic overflow occurred during a balance or supply calculation.

**Common cause:** Minting or transferring an amount that would push a balance or the total supply past `i128::MAX`.

**Resolution:** Ensure amounts are within safe bounds. This error should not occur under normal usage; if it does, it indicates a logic error in the calling contract.

---

### `InvalidNonce` (code 8)

**Description:** The nonce supplied to `approve_with_signature` does not match the owner's current expected nonce.

**Common cause:** Replaying an already-consumed permit message, or submitting permits out of order.

**Resolution:** Call `permit_nonce(owner)` to fetch the current nonce and have the owner sign a fresh message with it.

---

### `PermitExpired` (code 9)

**Description:** The current ledger is already past the `expiry_ledger` in the signed permit message.

**Common cause:** Submitting a permit too long after it was signed.

**Resolution:** Have the owner sign a new permit with a later `expiry_ledger`.

---

### `PermitSignerNotSet` (code 10)

**Description:** `approve_with_signature` was called for an `owner` who has never registered a permit signing key.

**Common cause:** Skipping the one-time `set_permit_signer` call before attempting signature-based approvals.

**Resolution:** Have the owner call `set_permit_signer` with their ed25519 public key before any spender submits a permit on their behalf.

---

## Escrow Contract — `EscrowError`

### `NotAuthorized` (code 1)

**Description:** The caller is not permitted to invoke this function.

**Common cause:** A party calling a function reserved for another role — e.g., the buyer calling `mark_delivered`, or a non-arbiter calling `resolve_dispute`.

**Resolution:** Verify which address is allowed to call each function. Refer to the contract docs for role requirements per function.

---

### `InvalidState` (code 2)

**Description:** The escrow is not in the required lifecycle state for this operation.

**Common cause:** Calling `fund` on an already-funded escrow, or calling `approve_delivery` before the escrow has been marked as delivered.

**Resolution:** Check the current escrow state with `get_state()` before calling state-dependent functions. Follow the expected lifecycle: `Created → Funded → Delivered → Completed`.

---

### `DeadlinePassed` (code 3)

**Description:** The escrow deadline has already elapsed; the operation is no longer valid.

**Common cause:** Attempting to fund or interact with an escrow after its deadline ledger has passed.

**Resolution:** Check `is_deadline_passed()` before interacting. If the deadline has passed, only refund-related flows are available.

---

### `DeadlineNotReached` (code 4)

**Description:** The deadline has not yet passed; a premature refund or timeout claim was attempted.

**Common cause:** Calling `request_refund` before the escrow deadline has been reached.

**Resolution:** Wait until the deadline ledger has passed before requesting a timeout-based refund. Use `is_deadline_passed()` to check.

---

### `AlreadyInitialized` (code 5)

**Description:** `initialize` was called on an escrow that is already set up.

**Common cause:** Calling `initialize` more than once on the same contract instance.

**Resolution:** Only call `initialize` once, right after deployment. Guard against re-initialization in your integration code.

---

### `NotInitialized` (code 6)

**Description:** An operation was attempted before the escrow was initialized.

**Common cause:** Calling any escrow function before `initialize` has been invoked.

**Resolution:** Always call `initialize` first with buyer, seller, arbiter, token, amount, and deadline parameters.

---

### `InsufficientFunds` (code 7)

**Description:** The buyer's token balance is too low to cover the escrowed amount.

**Common cause:** Calling `fund` when the buyer does not hold enough tokens, or has not approved the escrow contract to spend on their behalf.

**Resolution:** Ensure the buyer has called `approve` on the token contract for at least the escrow amount, and that their balance is sufficient.

---

### `InvalidAmount` (code 8)

**Description:** The specified escrow amount is zero or otherwise invalid.

**Common cause:** Passing `0` as the amount during `initialize`, or calling `update_amount` with a non-positive value.

**Resolution:** Always provide a positive, non-zero amount. Validate inputs before calling the contract.

---

### `InvalidParties` (code 9)

**Description:** Buyer, seller, or arbiter addresses are invalid or conflict with each other.

**Common cause:** Passing the same address for two different roles (e.g., buyer and seller are the same), or providing an invalid address.

**Resolution:** Ensure buyer, seller, and arbiter are three distinct, valid addresses before calling `initialize`.

---

## Wrapped-Token Contract — `WrappedTokenError`

### `AlreadyInitialized` (code 1)

**Description:** `initialize` was called on a contract that has already been set up.

**Common cause:** Calling `initialize` more than once on the same deployed contract instance.

**Resolution:** `initialize` should only be called once, immediately after deployment.

---

### `NotInitialized` (code 2)

**Description:** An operation was attempted before the contract was initialized.

**Common cause:** Invoking `wrap`, `unwrap`, or `get_reserve_balance` before `initialize` has been called.

**Resolution:** Call `initialize` with the admin, wrapped token, underlying token, and optional per-address cap before any other interaction.

---

### `Unauthorized` (code 3)

**Description:** The caller is not permitted to perform this action, or the contract is currently paused.

**Common cause:** Calling `wrap` or `unwrap` while the contract is paused (only possible when built with the `pausable` feature).

**Resolution:** Check contract state before retrying; wait for the admin to call `unpause()`.

---

### `InvalidAmount` (code 4)

**Description:** The amount is zero or negative, or `max_wrap_per_address` was set to zero or negative at `initialize`.

**Common cause:** Passing `0` or a negative value as an amount to `wrap`/`unwrap`, or an invalid cap to `initialize`.

**Resolution:** Validate that amounts are positive integers. If setting a cap, ensure it is greater than zero (or pass `None` for uncapped).

---

### `InsufficientBalance` (code 5)

**Description:** The caller does not hold enough wrapped tokens to complete the requested `unwrap`.

**Common cause:** Attempting to unwrap more than the wrapped token balance held by the caller.

**Resolution:** Check the wrapped token balance before calling `unwrap`.

---

### `InsufficientReserve` (code 6)

**Description:** The contract does not hold enough of the underlying asset to honor an `unwrap`.

**Common cause:** The underlying reserve has been depleted relative to the wrapped supply — this should never happen under normal 1:1-backed operation; see the reserve-backing invariant in the [monitoring guide](monitoring.md).

**Resolution:** Investigate immediately; this indicates a broken invariant between `get_total_wrapped()` and `get_reserve_balance()`.

---

### `MaxWrapExceeded` (code 7)

**Description:** Wrapping this amount would push the caller's cumulative wrapped total past the `max_wrap_per_address` cap set at `initialize`.

**Common cause:** A single address attempting to wrap more than the configured concentration-risk limit, either in one call or across multiple calls.

**Resolution:** Check `wrapped_by(address)` and `max_wrap_per_address()` before calling `wrap`, and keep cumulative wraps within the cap.
