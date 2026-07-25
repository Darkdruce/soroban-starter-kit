#!/usr/bin/env node
/**
 * Example: check and handle expiring token allowances using allowance_expiry().
 *
 * Demonstrates:
 *   1. Setting an allowance with an expiration ledger.
 *   2. Querying allowance_expiry() to check the expiration ledger.
 *   3. Detecting an expired allowance and renewing it.
 *
 * Prerequisites:
 *   npm install @stellar/stellar-sdk
 *   TOKEN_CONTRACT_ID=<id> node examples/typescript/allowance_expiry.js
 */

const {
  Keypair,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
  Contract,
  nativeToScVal,
  Address,
  xdr,
} = require('@stellar/stellar-sdk');

const RPC_URL = process.env.SOROBAN_RPC_URL || 'http://localhost:8000/soroban/rpc';
const NETWORK_PASSPHRASE = process.env.NETWORK_PASSPHRASE || 'Standalone Network ; February 2017';
const TOKEN_CONTRACT_ID = process.env.TOKEN_CONTRACT_ID;

if (!TOKEN_CONTRACT_ID) {
  console.error('Set TOKEN_CONTRACT_ID environment variable.');
  process.exit(1);
}

const server = new SorobanRpc.Server(RPC_URL, { allowHttp: true });

/** Build, sign, and submit a transaction; poll until confirmed. */
async function invokeContract(sourceKeypair, contractId, method, args) {
  const account = await server.getAccount(sourceKeypair.publicKey());
  const contract = new Contract(contractId);
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const prepared = await server.prepareTransaction(tx);
  prepared.sign(sourceKeypair);

  const result = await server.sendTransaction(prepared);
  if (result.status === 'ERROR') throw new Error(`Transaction failed: ${JSON.stringify(result)}`);

  let response = result;
  while (response.status === 'PENDING' || response.status === 'NOT_FOUND') {
    await new Promise((r) => setTimeout(r, 1000));
    response = await server.getTransaction(result.hash);
  }
  return response;
}

/**
 * Simulate a read-only call and return the decoded return value.
 * Uses simulateTransaction so no fee is spent.
 */
async function simulateCall(sourceKeypair, contractId, method, args) {
  const account = await server.getAccount(sourceKeypair.publicKey());
  const contract = new Contract(contractId);
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (!sim.result) return null;
  return sim.result.retval;
}

/** Return the current ledger sequence from the RPC server. */
async function getCurrentLedger() {
  const { sequence } = await server.getLatestLedger();
  return sequence;
}

async function main() {
  const admin = Keypair.random();
  const owner = Keypair.random();
  const spender = Keypair.random();

  // Fund accounts via friendbot (local node only).
  for (const kp of [admin, owner, spender]) {
    await fetch(`http://localhost:8000/friendbot?addr=${kp.publicKey()}`);
    console.log(`Funded ${kp.publicKey().slice(0, 8)}…`);
  }

  // Mint tokens to owner.
  const MINT_AMOUNT = 1_000_000n;
  await invokeContract(admin, TOKEN_CONTRACT_ID, 'mint', [
    new Address(owner.publicKey()).toScVal(),
    nativeToScVal(MINT_AMOUNT, { type: 'i128' }),
  ]);
  console.log(`\nMinted ${MINT_AMOUNT} tokens to owner.`);

  // ── Step 1: Approve with a short-lived expiration ──────────────────────────
  const currentLedger = await getCurrentLedger();
  // Expire after 100 ledgers (~8 minutes on mainnet).
  const SHORT_EXPIRY = currentLedger + 100;
  const ALLOWANCE_AMOUNT = 500_000n;

  await invokeContract(owner, TOKEN_CONTRACT_ID, 'approve', [
    new Address(owner.publicKey()).toScVal(),
    new Address(spender.publicKey()).toScVal(),
    nativeToScVal(ALLOWANCE_AMOUNT, { type: 'i128' }),
    nativeToScVal(BigInt(SHORT_EXPIRY), { type: 'u32' }),
  ]);
  console.log(`\nApproved ${ALLOWANCE_AMOUNT} tokens, expiring at ledger ${SHORT_EXPIRY}.`);

  // ── Step 2: Query allowance_expiry() ──────────────────────────────────────
  const expiryScVal = await simulateCall(owner, TOKEN_CONTRACT_ID, 'allowance_expiry', [
    new Address(owner.publicKey()).toScVal(),
    new Address(spender.publicKey()).toScVal(),
  ]);

  let expirationLedger = null;
  if (expiryScVal && expiryScVal.switch().name !== 'scvVoid') {
    // allowance_expiry returns Option<u32>: Some(ledger) or None (expired/absent).
    expirationLedger = expiryScVal.value()?.value()?.toNumber?.() ?? null;
  }

  if (expirationLedger === null) {
    console.log('\nAllowance is already expired or does not exist.');
  } else {
    console.log(`\nAllowance expires at ledger: ${expirationLedger}`);
    const ledgersRemaining = expirationLedger - currentLedger;
    console.log(`Current ledger: ${currentLedger}  →  ${ledgersRemaining} ledgers remaining`);

    // ── Step 3: Renew if expiring soon (within 50 ledgers) ──────────────────
    const RENEW_THRESHOLD = 50;
    if (ledgersRemaining <= RENEW_THRESHOLD) {
      console.log('\nAllowance expiring soon — renewing for another 1000 ledgers…');
      const NEW_EXPIRY = currentLedger + 1000;
      await invokeContract(owner, TOKEN_CONTRACT_ID, 'approve', [
        new Address(owner.publicKey()).toScVal(),
        new Address(spender.publicKey()).toScVal(),
        nativeToScVal(ALLOWANCE_AMOUNT, { type: 'i128' }),
        nativeToScVal(BigInt(NEW_EXPIRY), { type: 'u32' }),
      ]);
      console.log(`Allowance renewed. New expiry: ledger ${NEW_EXPIRY}`);
    } else {
      console.log('Allowance is still valid — no renewal needed.');
    }
  }

  // ── Simulate an expired allowance ─────────────────────────────────────────
  // Approve with expiry = currentLedger - 1 (already expired) to show the
  // "expired" handling path. In practice this would be detected after time passes.
  console.log('\n--- Simulating already-expired allowance ---');
  if (currentLedger > 0) {
    await invokeContract(owner, TOKEN_CONTRACT_ID, 'approve', [
      new Address(owner.publicKey()).toScVal(),
      new Address(spender.publicKey()).toScVal(),
      nativeToScVal(100n, { type: 'i128' }),
      nativeToScVal(BigInt(currentLedger - 1), { type: 'u32' }),
    ]);
  }

  const expiredScVal = await simulateCall(owner, TOKEN_CONTRACT_ID, 'allowance_expiry', [
    new Address(owner.publicKey()).toScVal(),
    new Address(spender.publicKey()).toScVal(),
  ]);

  const isExpired =
    !expiredScVal || expiredScVal.switch().name === 'scvVoid';

  if (isExpired) {
    console.log('Allowance has expired (allowance_expiry returned None).');
    console.log('Re-approving with fresh expiry…');
    const FRESH_EXPIRY = currentLedger + 500;
    await invokeContract(owner, TOKEN_CONTRACT_ID, 'approve', [
      new Address(owner.publicKey()).toScVal(),
      new Address(spender.publicKey()).toScVal(),
      nativeToScVal(ALLOWANCE_AMOUNT, { type: 'i128' }),
      nativeToScVal(BigInt(FRESH_EXPIRY), { type: 'u32' }),
    ]);
    console.log(`Re-approved. New expiry: ledger ${FRESH_EXPIRY}`);
  }

  console.log('\nAllowance expiry example complete.');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
