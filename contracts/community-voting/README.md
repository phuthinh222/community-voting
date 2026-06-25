# Community Voting Portal

A transparent, on-chain voting system built on **Stellar/Soroban**.

Community groups, student clubs, and small organizations often lack a trustworthy way to run votes. Results can be altered and the process is opaque. This contract stores every vote permanently on the Stellar blockchain — anyone can verify the outcome.

## Why Stellar

- Transaction fees under $0.001
- 3–5 second finality
- Soroban smart contracts with Rust type safety
- Built-in event system for frontend transparency

## Live Deployment — Testnet

| | |
|---|---|
| **Contract ID** | `CCDICCSVB7HS55Z32F4JTMAHCAMDLYKZAMLUT3Z4BQPMBMN2XM3T4S6A` |
| **Explorer** | [View on Stellar Expert ↗](https://stellar.expert/explorer/testnet/contract/CCDICCSVB7HS55Z32F4JTMAHCAMDLYKZAMLUT3Z4BQPMBMN2XM3T4S6A) |
| **Deploy TX** | [67006dae... ↗](https://stellar.expert/explorer/testnet/tx/67006daed22769b52f30485f3d3ca31a0c69482dd209a758837cdd55c620267b) |
| **create_proposal TX** | [4960f566... ↗](https://stellar.expert/explorer/testnet/tx/4960f566e693154b2368fcf5e0a093389a3a5c86f75cef2c90beca347d0b3763) |
| **vote TX** | [b3144beb... ↗](https://stellar.expert/explorer/testnet/tx/b3144beb134a2d090032aae2844ea428fd0552fe7fa1094ddbe12544ca847a56) |

## Contract Functions

| Function | Description |
|---|---|
| `initialize(admin)` | Set the contract admin (one-time) |
| `create_proposal(caller, title, description, deadline)` | Create a new proposal, returns proposal ID |
| `proposal_count()` | Get total number of proposals |
| `get_proposal(proposal_id)` | Fetch full proposal data |
| `vote(voter, proposal_id, vote_yes)` | Cast a YES or NO vote |
| `has_voted(proposal_id, voter)` | Check if an address has voted |
| `result(proposal_id)` | Returns `"PASSED"` / `"REJECTED"` / `"TIED"` |
| `close_proposal(caller, proposal_id)` | Admin closes voting on a proposal |

## Proposal Struct

```rust
pub struct Proposal {
    pub id:          u64,
    pub title:       String,
    pub description: String,
    pub yes_count:   u64,
    pub no_count:    u64,
    pub active:      bool,
    pub created_at:  u64,  // ledger timestamp
    pub deadline:    u64,  // unix timestamp
}
```

## On-Chain Events

| Event | Trigger | Data |
|---|---|---|
| `"create"` | New proposal created | `proposal_id: u64` |
| `"vote"` | Vote cast | `(proposal_id: u64, vote_yes: bool)` |

## Tech Stack

- **Smart Contract**: Rust, Soroban SDK v22, WebAssembly
- **Frontend**: Vanilla HTML/CSS/JS + `@stellar/stellar-sdk`
- **Wallet**: Freighter browser extension
- **Network**: Stellar Testnet

## Build & Test

```bash
# Run tests (9 tests)
cargo test

# Build WASM (~6KB)
stellar contract build
```

## Deploy

```bash
# 1. Create identity
stellar keys generate student --network testnet --fund

# 2. Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/community_voting.wasm \
  --source-account student \
  --network testnet

# 3. Initialize
stellar contract invoke \
  --id <CONTRACT_ID> --source-account student --network testnet \
  -- initialize --admin $(stellar keys address student)

# 4. Create a proposal
stellar contract invoke \
  --id <CONTRACT_ID> --source-account student --network testnet \
  -- create_proposal \
  --caller $(stellar keys address student) \
  --title "My first proposal" \
  --description "Should we build X?" \
  --deadline 9999999999

# 5. Vote
stellar contract invoke \
  --id <CONTRACT_ID> --source-account student --network testnet \
  -- vote \
  --voter $(stellar keys address student) \
  --proposal_id 1 \
  --vote_yes true
```

## Frontend

Open `../../frontend/index.html` in a browser with Freighter installed and switched to Testnet.

Features: connect wallet · create proposals · vote YES/NO · live result bar · transaction links.

## Test Coverage

| Test | What it verifies |
|---|---|
| `test_voting_flow` | Happy path: create → vote → check counts |
| `test_duplicate_vote` | Same voter panics on second vote |
| `test_closed_proposal_vote` | Voting on closed proposal panics |
| `test_multiple_proposals` | Independent proposals don't interfere |
| `test_close_not_authorized` | Non-admin cannot close proposals |
| `test_voting_ended` | Vote after deadline panics |
| `test_result` | PASSED when YES > NO |
| `test_result_rejected` | REJECTED when NO > YES |
| `test_has_voted` | Returns false before, true after voting |

## WASM Size

```
6,227 bytes  (limit: 64KB — using < 10%)
```
