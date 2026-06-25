# Community Voting Portal

## Problem

Community groups and student clubs have no transparent way to run votes — results can be silently altered and no one can verify them.

## Solution

A Soroban smart contract that stores every vote permanently on Stellar, so anyone can audit the outcome publicly in real time.

## Why Stellar

Stellar's sub-cent transaction fees and 5-second finality make on-chain voting practical for small communities that can't afford gas-heavy chains.

## Target User

Student clubs, neighborhood groups, and small organizations that need trustworthy, verifiable voting without managing their own infrastructure.

## Live Demo

- Network: Stellar Testnet
- **Contract ID**: `CCDICCSVB7HS55Z32F4JTMAHCAMDLYKZAMLUT3Z4BQPMBMN2XM3T4S6A`
- **Transaction (create_proposal)**: [4960f566...](https://stellar.expert/explorer/testnet/tx/4960f566e693154b2368fcf5e0a093389a3a5c86f75cef2c90beca347d0b3763)
- **Transaction (vote)**: [b3144beb...](https://stellar.expert/explorer/testnet/tx/b3144beb134a2d090032aae2844ea428fd0552fe7fa1094ddbe12544ca847a56)

## How to Run

1. Clone: `git clone https://github.com/phuthinh222/community.git`
2. Build: `cd contracts/community-voting && stellar contract build`
3. Test: `cargo test`
4. Deploy: `stellar contract deploy --wasm target/wasm32v1-none/release/community_voting.wasm --source-account student --network testnet`
5. Frontend: open `frontend/index.html` in browser with Freighter installed (switch Freighter to Testnet)

## Tech Stack

- Smart Contract: Rust / Soroban SDK v22
- Frontend: HTML / JavaScript / @stellar/stellar-sdk
- Wallet: Freighter
- Network: Stellar Testnet

## Team

- Phu Thinh | [@your\_telegram] | truongdophuthinh@gmail.com | [University + Year]
