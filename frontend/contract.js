'use strict';

// ── CONTRACT CONFIG ───────────────────────────────────────────────────────────
const CONTRACT_ID  = 'CCDICCSVB7HS55Z32F4JTMAHCAMDLYKZAMLUT3Z4BQPMBMN2XM3T4S6A';
const RPC_URL      = 'https://soroban-testnet.stellar.org';
const PASSPHRASE   = 'Test SDF Network ; September 2015';
const EXPLORER     = 'https://stellar.expert/explorer/testnet';
const FALLBACK_KEY = 'GDGQZXYCNQBQRQQHKHYGBDGA4KIBXSTPDJMI5H64LFBBU4ZGQCWW7QG6';

// ── SDK IMPORTS (loaded via <script> in index.html) ───────────────────────────
// StellarSdk is exposed globally from @stellar/stellar-sdk
const {
  SorobanRpc,
  Contract,
  TransactionBuilder,
  BASE_FEE,
  Address,
  nativeToScVal,
  scValToNative,
} = StellarSdk;

// ── STATE ─────────────────────────────────────────────────────────────────────
let walletAddress = null;

// ── WALLET CONNECTION (Freighter) ─────────────────────────────────────────────
async function connectWallet() {
  if (!window.freighter) {
    throw new Error('Freighter wallet not installed — install from freighter.app');
  }
  if (window.freighter.setAllowed) {
    await window.freighter.setAllowed();
  }
  const res = await window.freighter.getAddress();
  walletAddress = res.address || res;
  return walletAddress;
}

// ── SOROBAN RPC HELPERS ───────────────────────────────────────────────────────
function server()   { return new SorobanRpc.Server(RPC_URL); }
function contract() { return new Contract(CONTRACT_ID); }

async function readContract(funcName, ...args) {
  const srv  = server();
  const src  = walletAddress || FALLBACK_KEY;
  const acct = await srv.getAccount(src);
  const tx   = new TransactionBuilder(acct, { fee: BASE_FEE, networkPassphrase: PASSPHRASE })
    .addOperation(contract().call(funcName, ...args))
    .setTimeout(30)
    .build();
  const sim = await srv.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(sim)) throw new Error(sim.error);
  return scValToNative(sim.result.retval);
}

async function writeContract(funcName, ...args) {
  if (!walletAddress) throw new Error('Connect wallet first');
  if (!window.freighter) throw new Error('Freighter not installed');
  const srv  = server();
  const acct = await srv.getAccount(walletAddress);
  const tx   = new TransactionBuilder(acct, { fee: BASE_FEE, networkPassphrase: PASSPHRASE })
    .addOperation(contract().call(funcName, ...args))
    .setTimeout(30)
    .build();
  const prepared = await srv.prepareTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(prepared)) throw new Error(prepared.error);

  const { signedTxXdr } = await window.freighter.signTransaction(prepared.toXDR(), {
    networkPassphrase: PASSPHRASE,
  });
  const signedTx = StellarSdk.TransactionBuilder.fromXDR(signedTxXdr, PASSPHRASE);
  const resp = await srv.sendTransaction(signedTx);
  if (resp.status === 'ERROR') throw new Error(resp.errorResult?.toString() || 'TX error');

  for (let i = 0; i < 24; i++) {
    await new Promise(r => setTimeout(r, 1500));
    const txr = await srv.getTransaction(resp.hash);
    if (txr.status === 'SUCCESS') return { hash: resp.hash };
    if (txr.status === 'FAILED')  throw new Error('Transaction failed on-chain');
  }
  throw new Error('Transaction timeout');
}

// ── CONTRACT FUNCTIONS ────────────────────────────────────────────────────────

// initialize(admin: Address) — one-time setup, sets contract admin
async function initialize(admin) {
  return writeContract('initialize', new Address(admin).toScVal());
}

// create_proposal(caller, title, description, deadline) → proposal_id: u64
async function createProposal(title, description, deadlineDays) {
  const deadline = BigInt(Math.floor(Date.now() / 1000) + deadlineDays * 86400);
  return writeContract(
    'create_proposal',
    new Address(walletAddress).toScVal(),
    nativeToScVal(title,       { type: 'string' }),
    nativeToScVal(description, { type: 'string' }),
    nativeToScVal(deadline,    { type: 'u64' }),
  );
}

// proposal_count() → u64
async function proposalCount() {
  return readContract('proposal_count');
}

// get_proposal(proposal_id) → Proposal struct
async function getProposal(id) {
  return readContract('get_proposal', nativeToScVal(BigInt(id), { type: 'u64' }));
}

// vote(voter, proposal_id, vote_yes) — cast YES or NO
async function vote(proposalId, voteYes) {
  return writeContract(
    'vote',
    new Address(walletAddress).toScVal(),
    nativeToScVal(BigInt(proposalId), { type: 'u64' }),
    nativeToScVal(voteYes,            { type: 'bool' }),
  );
}

// has_voted(proposal_id, voter) → bool
async function hasVoted(proposalId, voter) {
  return readContract(
    'has_voted',
    nativeToScVal(BigInt(proposalId), { type: 'u64' }),
    new Address(voter).toScVal(),
  );
}

// result(proposal_id) → "PASSED" | "REJECTED" | "TIED"
async function result(proposalId) {
  return readContract('result', nativeToScVal(BigInt(proposalId), { type: 'u64' }));
}

// close_proposal(caller, proposal_id) — admin only
async function closeProposal(proposalId) {
  return writeContract(
    'close_proposal',
    new Address(walletAddress).toScVal(),
    nativeToScVal(BigInt(proposalId), { type: 'u64' }),
  );
}
