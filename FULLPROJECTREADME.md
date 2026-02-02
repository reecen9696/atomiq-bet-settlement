# Atomik Blockchain Casino Platform

A provably-fair, non-custodial blockchain casino platform combining a high-performance custom blockchain with Solana smart contracts for real-money settlement.

**Key Features:**

- ⚡ **10-20ms game finalization** using DirectCommit consensus
- 🎲 **Provably fair gaming** via Schnorrkel VRF (Verifiable Random Functions)
- 🔐 **Non-custodial** - users maintain control of funds in Solana PDAs
- 💨 **Gasless betting** - approve once, bet multiple times without signatures
- 🔄 **Real-time updates** - WebSocket broadcasts for instant UX
- 🚀 **High throughput** - 10K+ TPS capable blockchain layer

---

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Component Overview](#component-overview)
3. [Data Flow](#data-flow)
4. [Smart Contracts](#smart-contracts)
5. [API Reference](#api-reference)
6. [WebSocket Events](#websocket-events)
7. [Getting Started](#getting-started)
8. [Configuration](#configuration)
9. [Technical Details](#technical-details)
10. [Current Status](#current-status)

---

## System Architecture

The platform consists of three main components:

```
┌─────────────────────────────────────────────────────────────┐
│                    USER (Web Frontend)                       │
│                                                              │
│  • React + TypeScript test UI                               │
│  • Solana wallet integration (Privy)                        │
│  • WebSocket for real-time updates                          │
└─────────────────────────────────────────────────────────────┘
                            ↓
        ┌───────────────────┴───────────────────┐
        ↓                                       ↓
┌──────────────────┐                   ┌──────────────────┐
│   Blockchain     │←─── Settlement────│   Transaction    │
│  Gaming Engine   │      API          │    Processor     │
│                  │                   │                  │
│ • VRF outcomes   │                   │ • Polls pending  │
│ • RocksDB store  │                   │ • Solana TX      │
│ • WebSocket push │                   │ • Workers pool   │
└──────────────────┘                   └──────────────────┘
        ↓                                       ↓
┌──────────────────┐                   ┌──────────────────┐
│    Local DB      │                   │  Solana Vault    │
│   (RocksDB)      │                   │   Program        │
│                  │                   │  (Smart Contract)│
│ • Game results   │                   │                  │
│ • VRF proofs     │                   │ • User vaults    │
│ • Settlement     │                   │ • Allowances     │
└──────────────────┘                   └──────────────────┘
```

### Technology Stack

| Component                 | Technologies                                               |
| ------------------------- | ---------------------------------------------------------- |
| **Blockchain**            | Rust, HotStuff-rs consensus, RocksDB, Schnorrkel VRF, Axum |
| **Transaction Processor** | Rust, Axum, PostgreSQL, Redis, Solana Client               |
| **Smart Contracts**       | Anchor 0.30.1, Solana SDK 1.17+                            |
| **Frontend**              | React 18, TypeScript, Vite, Privy, @solana/web3.js         |

---

## Component Overview

### 1. Blockchain Gaming Engine

**Location:** `blockchain/`  
**Purpose:** High-performance gaming engine with provably fair outcomes

**Core Responsibilities:**

- Process game bets with VRF-based randomness
- Finalize games in 10-20ms using DirectCommit consensus
- Store game results and VRF proofs in RocksDB
- Expose REST API for bet placement and game queries
- Broadcast real-time events via WebSocket
- Provide Settlement API for transaction processor integration

**Key Components:**

| File                               | Purpose                                               |
| ---------------------------------- | ----------------------------------------------------- |
| `src/main_unified.rs`              | Main entry point with CLI                             |
| `src/direct_commit.rs`             | DirectCommit consensus engine (1000ms block interval) |
| `src/blockchain_game_processor.rs` | Game processing with VRF integration                  |
| `src/games/vrf_engine.rs`          | Schnorrkel VRF for provably fair randomness           |
| `src/game_store.rs`                | RocksDB storage for game results                      |
| `src/api/games.rs`                 | Game API endpoints (coinflip, dice, etc.)             |
| `src/api/settlement.rs`            | Settlement API for processor integration              |
| `src/api/websocket.rs`             | WebSocket event broadcasting                          |
| `src/finalization.rs`              | Event-driven finalization system                      |

**Data Storage:**

RocksDB keys:

- `block:height:150000` → Full block data
- `tx_idx:12345` → Transaction lookup (height:index pointer)
- `game:result:tx:12345` → Complete game result with VRF proof
- `settlement:pending:12345` → Lightweight settlement summary
- `game:index:recent:inv_height:tx_id` → Recent games index

**Storage Optimizations:**

- 73% storage reduction via intelligent indexing
- Blocks stored by height only (not duplicated by hash)
- Transaction index as pointer (height:index) instead of full TX duplication

### 2. Transaction Processor

**Location:** `transaction-processor/`  
**Purpose:** Bridges blockchain gaming results with Solana for real-money settlement

**Architecture:**

```
Backend API (Port 3001)    Processor Service
       ↓                          ↓
  PostgreSQL ←────────────→ Coordinator
   + Redis                        ↓
                          ┌───────┴───────┐
                          ↓       ↓       ↓
                       Worker  Worker  Worker
                          ↓       ↓       ↓
                        Solana DevNet
```

**Core Responsibilities:**

- Poll blockchain Settlement API for pending games
- Build Solana transactions (`spend_from_allowance` + `payout`)
- Submit transactions to Solana RPC with retry logic
- Update blockchain settlement status with optimistic locking
- Prevent duplicate processing via coordinator pattern

**Key Components:**

| Directory                                     | Purpose                                    |
| --------------------------------------------- | ------------------------------------------ |
| `services/backend/`                           | REST API for bet creation, user management |
| `services/processor/`                         | Settlement coordinator + worker pool       |
| `services/processor/src/settlement_worker.rs` | Solana transaction builder and submitter   |
| `services/processor/src/worker_pool/`         | Parallel processing with channels          |
| `services/shared/`                            | Shared data models and types               |

**Processing Modes:**

1. **Coordinator Mode** ✅ (Production)
   - Central coordinator fetches pending settlements
   - Distributes work to 4 parallel workers via channels
   - Prevents duplicate processing
   - Batch size: 3-12 settlements per batch

2. **Legacy Polling Mode** ⚠️ (Deprecated)
   - Each worker polls independently
   - Risk of race conditions

### 3. Solana Smart Contracts

**Location:** `blockchain/solana-playground-deploy/programs/vault/`  
**Program ID:** `BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ` (DevNet)  
**Purpose:** Non-custodial vault system for user funds on Solana

**Account Types:**

| Account          | Seeds                                 | Purpose                            |
| ---------------- | ------------------------------------- | ---------------------------------- |
| **Vault**        | `[b"vault", casino, user]`            | User's SOL vault (PDA)             |
| **CasinoVault**  | `[b"casino-vault", casino]`           | Casino's fund pool (program-owned) |
| **Allowance**    | `[b"allowance", user, casino, nonce]` | Gasless betting approval           |
| **Casino**       | `[b"casino"]`                         | Configuration account              |
| **ProcessedBet** | `[b"processed-bet", bet_id]`          | Deduplication mechanism            |

**See [Smart Contracts](#smart-contracts) section for detailed instruction documentation.**

---

## Data Flow

### Complete User Journey: Place Bet → Receive Winnings

```
1. USER SETUP (One-time)
   ├─→ Frontend: Connect Solana wallet (Phantom, Solflare, etc.)
   ├─→ Smart Contract: initialize_vault()
   ├─→ Smart Contract: deposit_sol(5 SOL)
   └─→ Smart Contract: approve_allowance(5 SOL, 10000 seconds)

2. PLACE BET
   ├─→ Frontend: POST /api/coinflip/play
   │   Request: {choice: "heads", amount: 1_000_000_000, player_address: "8x7Y..."}
   │
   ├─→ Blockchain API: Validate and create GameBet transaction
   │   └─→ Transaction Pool: Queue TX for processing
   │
   └─→ Response: {game_id: "uuid", status: "pending", transaction_id: 12345}

3. GAME PROCESSING (10-20ms)
   ├─→ DirectCommit Engine: Drain transaction pool (every 1000ms)
   │
   ├─→ Game Processor: Execute VRF for TX 12345
   │   ├─ Input: "tx-12345:CoinFlip:8x7Y:Heads:block_hash:...:height:150000"
   │   ├─ VRF Sign: Generate deterministic output + proof
   │   ├─ Outcome: vrf_output[0] % 2 == 0 ? Heads : Tails
   │   └─ Result: Win (payout: 2 SOL) or Loss (payout: 0)
   │
   ├─→ Block Creation:
   │   ├─ Height: 150000
   │   ├─ Block hash: SHA256(height + prev + tx_root + state + timestamp)
   │   └─ Transactions: [TX 12345, ...]
   │
   ├─→ RocksDB Commit:
   │   ├─ game:result:tx:12345 → {outcome: Win, payout: 2000000000, vrf_proof: [...]}
   │   ├─ settlement:pending:12345 → {tx_id, player, amount, version: 1}
   │   └─ settlement_status: PendingSettlement
   │
   └─→ WebSocket Broadcast:
       ├─ {type: "new_block", height: 150000, transactions: ["12345"]}
       └─ {type: "casino_win", amount_won: 2.0, tx_id: "12345"}

4. SETTLEMENT COORDINATION (Every 10 seconds)
   ├─→ Coordinator: GET /api/settlement/pending?limit=200
   │   Response: [
   │     {transaction_id: 12345, outcome: "Win", payout: 2000000000, version: 1, ...}
   │   ]
   │
   ├─→ Coordinator: Group by outcome
   │   ├─ Wins: [12345, 67890, ...]  → Batch 1
   │   └─ Losses: [23456, ...]       → Batch 2
   │
   └─→ Coordinator: Distribute to workers via channels (round-robin)

5. SETTLEMENT PROCESSING (Parallel Workers)
   ├─→ Worker #1: Receive batch [12345]
   │
   ├─→ Step 1: Claim work (Optimistic Locking)
   │   POST /api/settlement/games/12345
   │   Request: {status: "SubmittedToSolana", expected_version: 1}
   │   Response: {success: true, new_version: 2}
   │
   ├─→ Step 2: Build Solana Transaction
   │   For WIN:
   │     └─ payout(casino_vault → user_vault, 2 SOL, bet_id: "bet-12345")
   │   For LOSS:
   │     └─ spend_from_allowance(user_vault → casino_vault, 1 SOL, bet_id: "bet-12345")
   │
   ├─→ Step 3: Submit to Solana
   │   └─ solana_client.send_and_confirm_transaction(tx)
   │   └─ Signature: "5X7Y9mK5dQ3pR2nF1vS4wL..."
   │
   └─→ Step 4: Update Complete (INFINITE RETRY)
       POST /api/settlement/games/12345
       Request: {
         status: "SettlementComplete",
         solana_tx_id: "5X7Y...",
         expected_version: 2
       }
       Response: {success: true, new_version: 3}

6. FINAL STATE
   ├─→ Blockchain: settlement_status = SettlementComplete, version = 3
   ├─→ Solana: User vault balance increased by 2 SOL
   ├─→ Frontend: WebSocket receives update, displays "You won 2 SOL!"
   └─→ User: Can withdraw SOL from vault to wallet anytime
```

### Settlement Status Flow

```
PendingSettlement
    ↓
    ↓ (Coordinator fetches)
    ↓
SubmittedToSolana (Optimistic lock acquired)
    ↓
    ↓ (Solana TX confirmed)
    ↓
SettlementComplete ✅
```

**Error Handling:**

```
SettlementFailed (retry_count < 3)
    ↓
    ↓ (Exponential backoff: 5s, 10s, 15s)
    ↓
PendingSettlement (re-queued for retry)
    ↓
    ↓ (retry_count >= 3)
    ↓
SettlementFailedPermanent ⚠️ (Manual review needed)
```

---

## Smart Contracts

### Vault Program Architecture

**Framework:** Anchor 0.30.1  
**Chain:** Solana DevNet  
**Program ID:** `BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ`

### Account Structures

#### 1. Vault (User PDA)

```rust
#[account]
pub struct Vault {
    pub owner: Pubkey,        // User's wallet
    pub casino: Pubkey,       // Casino pubkey
    pub sol_balance: u64,     // Tracked SOL balance
    pub created_at: i64,
    pub last_activity: i64,
}

// Seeds: [b"vault", casino.key(), user.key()]
```

#### 2. CasinoVault (Program-Owned Account)

```rust
#[account]
pub struct CasinoVault {
    pub casino: Pubkey,       // Casino configuration
    pub sol_balance: u64,     // Tracked SOL balance
    pub created_at: i64,
    pub last_activity: i64,
}

// Seeds: [b"casino-vault", casino.key()]
```

**Why program-owned?** Enables direct lamports manipulation (~100 CU) instead of System Program CPI (~5,000 CU).

#### 3. Allowance (Gasless Betting)

```rust
#[account]
pub struct Allowance {
    pub user: Pubkey,
    pub casino: Pubkey,
    pub token_mint: Pubkey,   // System Program for native SOL
    pub amount: u64,          // Total approved
    pub spent: u64,           // Already spent
    pub expires_at: i64,      // Unix timestamp
    pub nonce: u64,           // For multiple allowances
    pub revoked: bool,
}

// Seeds: [b"allowance", user.key(), casino.key(), nonce.to_le_bytes()]
```

**Constraints:**

- Max amount: 10,000 SOL
- Max duration: 24 hours
- Rate limit: 100 approvals/hour (prevents spam)

#### 4. Casino (Configuration)

```rust
#[account]
pub struct Casino {
    pub authority: Pubkey,    // Admin
    pub processor: Pubkey,    // Authorized processor
    pub treasury: Pubkey,     // Profit withdrawal destination
    pub paused: bool,         // Emergency pause
    pub total_bets: u64,
    pub total_volume: u64,
}

// Seeds: [b"casino"]
```

#### 5. ProcessedBet (Deduplication)

```rust
#[account]
pub struct ProcessedBet {
    pub bet_id: String,
    pub user: Pubkey,
    pub amount: u64,
    pub processed_at: i64,
}

// Seeds: [b"processed-bet", bet_id.as_bytes()]
```

**Purpose:** Prevents replay attacks. Once created, same `bet_id` cannot be processed again.

### Instructions

#### Core User Instructions

##### initialize_vault

```rust
pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()>
```

**Purpose:** Create user's vault PDA  
**Accounts:** `[writable] vault, [signer] user, casino, system_program`  
**Effect:** Creates vault with 0 SOL balance

##### deposit_sol

```rust
pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()>
```

**Purpose:** User deposits SOL to their vault  
**Accounts:** `[writable] vault, [writable, signer] user, system_program`  
**Effect:** Transfers SOL from user wallet → vault, updates `sol_balance`

##### withdraw_sol

```rust
pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()>
```

**Purpose:** User withdraws SOL from vault to wallet  
**Accounts:** `[writable] vault, [writable, signer] user, system_program`  
**Effect:** Transfers SOL from vault → user wallet, updates `sol_balance`  
**Note:** Always available, even if casino is paused

##### approve_allowance_v2

```rust
pub fn approve_allowance_v2(
    ctx: Context<ApproveAllowanceV2>,
    amount: u64,
    duration: i64,
    token_mint: Pubkey,
    nonce: u64,
) -> Result<()>
```

**Purpose:** Approve processor to spend up to `amount` for `duration` seconds  
**Accounts:** `[writable] allowance, [signer] user, casino, vault, system_program`  
**Constraints:**

- `amount <= 10_000_000_000_000` (10,000 SOL)
- `duration <= 86_400` (24 hours)
- `vault.sol_balance >= amount`
- Rate limited per user

**Effect:** Creates allowance PDA with nonce-based deterministic address

##### revoke_allowance

```rust
pub fn revoke_allowance(ctx: Context<RevokeAllowance>) -> Result<()>
```

**Purpose:** User revokes allowance  
**Accounts:** `[writable] allowance, [signer] user`  
**Effect:** Sets `allowance.revoked = true`

#### Core Settlement Instructions

##### spend_from_allowance

```rust
pub fn spend_from_allowance(
    ctx: Context<SpendFromAllowance>,
    amount: u64,
    bet_id: String,
) -> Result<()>
```

**Purpose:** Processor spends from user vault using allowance (NO USER SIGNATURE)  
**Accounts:**

- `[writable] vault` (user's vault)
- `[writable] casino_vault`
- `[writable] allowance`
- `[writable] processed_bet` (PDA to create)
- `[signer] processor` (authorized processor only)
- `casino`, `clock`, `system_program`

**Validation:**

1. `processor.key() == casino.processor` (only authorized processor)
2. `!allowance.revoked`
3. `clock.unix_timestamp < allowance.expires_at`
4. `allowance.spent + amount <= allowance.amount`
5. `vault.sol_balance >= amount`
6. `amount >= 10_000_000` (min bet: 0.01 SOL)
7. `amount <= 1_000_000_000_000` (max bet: 1000 SOL)
8. `processed_bet` account doesn't exist (creates new, prevents duplicates)

**Effect:**

```rust
// Direct lamports manipulation (program-owned accounts)
**vault.to_account_info().try_borrow_mut_lamports()? -= amount;
**casino_vault.to_account_info().try_borrow_mut_lamports()? += amount;

vault.sol_balance -= amount;
casino_vault.sol_balance += amount;
allowance.spent += amount;

// Create ProcessedBet account (deduplication)
processed_bet.bet_id = bet_id;
processed_bet.amount = amount;
```

##### payout

```rust
pub fn payout(
    ctx: Context<Payout>,
    amount: u64,
    bet_id: String,
) -> Result<()>
```

**Purpose:** Casino pays winnings to user vault  
**Accounts:**

- `[writable] casino_vault`
- `[writable] vault` (user's vault)
- `[signer] processor`
- `casino`, `system_program`

**Validation:**

1. `processor.key() == casino.processor`
2. `!casino.paused`
3. `casino_vault.sol_balance >= amount`

**Effect:**

```rust
// Direct lamports manipulation
**casino_vault.to_account_info().try_borrow_mut_lamports()? -= amount;
**vault.to_account_info().try_borrow_mut_lamports()? += amount;

casino_vault.sol_balance -= amount;
vault.sol_balance += amount;
```

**Note:** Does NOT check for duplicate `bet_id` (payouts can happen multiple times, e.g., bonuses)

#### Admin Instructions

##### withdraw_casino_funds

```rust
pub fn withdraw_casino_funds(
    ctx: Context<WithdrawCasinoFunds>,
    amount: u64,
) -> Result<()>
```

**Purpose:** Admin withdraws casino profit  
**Accounts:** `[writable] casino_vault, [writable] treasury, [signer] authority, casino, system_program`  
**Effect:** Transfers SOL from casino_vault → treasury

##### pause_casino / unpause_casino

```rust
pub fn pause_casino(ctx: Context<PauseCasino>) -> Result<()>
```

**Purpose:** Emergency pause (stops all bets/payouts)  
**Accounts:** `[writable] casino, [signer] authority`  
**Effect:** Sets `casino.paused = true`

**Note:** User withdrawals always work, even when paused.

### Gasless Betting Mechanism

**Problem:** Users don't want to sign every bet (bad UX)  
**Solution:** ERC-20-style allowance pattern on Solana

**Flow:**

1. **One-time approval:** User signs `approve_allowance_v2(5 SOL, 10000 seconds)`
2. **Gasless bets:** Processor calls `spend_from_allowance` WITHOUT user signature
3. **Funds stay in user's vault:** Non-custodial (user maintains control)
4. **User can revoke anytime:** Call `revoke_allowance` or let it expire

**Security:**

- Time-bounded (max 24 hours)
- Amount-bounded (max 10,000 SOL)
- Revocable by user
- Rate limited (100 approvals/hour)
- Deduplication via `ProcessedBet` accounts

---

## API Reference

### Blockchain API (Port 8080)

Base URL: `http://localhost:8080`

#### Health & Status

##### GET /health

**Response:**

```json
{
  "status": "ok"
}
```

##### GET /status

**Response:**

```json
{
  "latest_block": 150000,
  "latest_block_hash": "a1b2c3...",
  "pending_transactions": 42,
  "consensus_mode": "DirectCommit"
}
```

##### GET /metrics

**Response:** Prometheus metrics format

```
# HELP blockchain_tps Transactions per second
# TYPE blockchain_tps gauge
blockchain_tps 1234.56
```

#### Block Explorer

##### GET /blocks?limit=50

**Query Params:**

- `limit` (optional): Max blocks to return (default: 50, max: 1000)

**Response:**

```json
{
  "blocks": [
    {
      "height": 150000,
      "hash": "a1b2c3...",
      "timestamp": 1738483200000,
      "transaction_count": 123
    }
  ]
}
```

##### GET /block/:height

**Response:**

```json
{
  "height": 150000,
  "hash": "a1b2c3...",
  "previous_hash": "9d8e7f...",
  "timestamp": 1738483200000,
  "transactions": [
    {
      "id": 12345,
      "sender": "8x7Y...",
      "timestamp": 1738483199500
    }
  ],
  "transactions_root": "5f6g7h...",
  "state_root": "1x2y3z..."
}
```

##### GET /tx/:tx_id

**Response:**

```json
{
  "id": 12345,
  "sender": "8x7Y...",
  "data": "...",
  "timestamp": 1738483199500,
  "nonce": 42,
  "block_height": 150000,
  "block_hash": "a1b2c3..."
}
```

#### Game Endpoints

##### POST /api/coinflip/play

**Request:**

```json
{
  "choice": "heads",
  "amount": 1000000000,
  "player_address": "8x7Y9mK5dQ3pR2nF1vS4wL6eT8hX9oP1aS3fG7jH"
}
```

**Response:**

```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "outcome": "heads",
  "won": true,
  "payout": 2000000000,
  "vrf_proof": "a1b2c3d4e5f6...",
  "vrf_output": "7h8i9j0k1l2m...",
  "transaction_id": 12345,
  "block_height": 150000,
  "timestamp": 1738483199500
}
```

##### GET /api/game/:game_id

**Response:**

```json
{
  "game_id": "550e8400-e29b-41d4-a716-446655440000",
  "transaction_id": 12345,
  "player_address": "8x7Y...",
  "game_type": "CoinFlip",
  "bet_amount": 1000000000,
  "player_choice": "heads",
  "outcome": "heads",
  "won": true,
  "payout": 2000000000,
  "vrf_proof": "a1b2c3...",
  "vrf_output": "7h8i9j...",
  "block_height": 150000,
  "settlement_status": "SettlementComplete",
  "solana_tx_id": "5X7Y9mK5dQ..."
}
```

##### GET /api/games/recent?limit=20&cursor=...

**Query Params:**

- `limit` (optional): Max games to return (default: 20, max: 100)
- `cursor` (optional): Pagination cursor from previous response

**Response:**

```json
{
  "games": [...],
  "next_cursor": "6f7574626f783a..."
}
```

##### GET /api/tokens

**Response:**

```json
{
  "tokens": [
    {
      "symbol": "SOL",
      "name": "Solana",
      "decimals": 9,
      "mint": "So11111111111111111111111111111111111111112"
    }
  ]
}
```

#### Verification Endpoints

##### POST /api/verify/vrf

**Request:**

```json
{
  "vrf_output": "7h8i9j0k1l2m...",
  "vrf_proof": "a1b2c3d4e5f6...",
  "public_key": "blockchain_pubkey_hex",
  "input_message": "tx-12345:CoinFlip:8x7Y:heads:block_hash:...:height:150000"
}
```

**Response:**

```json
{
  "valid": true
}
```

##### GET /api/verify/game/:game_id

**Response:**

```json
{
  "game_id": "550e8400-...",
  "valid": true,
  "vrf_verified": true,
  "details": {
    "input_message": "tx-12345:...",
    "vrf_output_matches": true,
    "outcome_derived_correctly": true
  }
}
```

#### Settlement API (Internal - For Transaction Processor)

##### GET /api/settlement/pending?limit=200&cursor=...

**Headers:** `X-API-Key: settlement-api-key-2026`

**Query Params:**

- `limit` (optional): Max settlements to return (default: 50, max: 500)
- `cursor` (optional): Hex-encoded RocksDB key for pagination

**Response:**

```json
{
  "games": [
    {
      "transaction_id": 12345,
      "player_address": "8x7Y...",
      "game_type": "CoinFlip",
      "bet_amount": 1000000000,
      "token": "SOL",
      "outcome": "Win",
      "payout": 2000000000,
      "vrf_proof": "a1b2c3...",
      "vrf_output": "7h8i9j...",
      "block_height": 150000,
      "version": 1,
      "retry_count": 0,
      "next_retry_after": null,
      "solana_tx_id": null
    }
  ],
  "next_cursor": "73657474:..."
}
```

**Notes:**

- Includes `PendingSettlement` and `SettlementFailed` (retry_count < 3, past retry_after)
- Excludes `SettlementComplete`, `SubmittedToSolana`, `SettlementFailedPermanent`
- Cursor-based pagination for large result sets

##### GET /api/settlement/games/:tx_id

**Headers:** `X-API-Key: settlement-api-key-2026`

**Response:**

```json
{
  "transaction_id": 12345,
  "settlement_status": "PendingSettlement",
  "version": 1,
  "solana_tx_id": null,
  "settlement_error": null,
  "retry_count": 0,
  "next_retry_after": null
}
```

##### POST /api/settlement/games/:tx_id

**Headers:** `X-API-Key: settlement-api-key-2026`

**Request:**

```json
{
  "status": "SubmittedToSolana",
  "expected_version": 1,
  "solana_tx_id": "5X7Y9mK5dQ...",
  "error_message": null
}
```

**Valid Status Values:**

- `SubmittedToSolana` - TX sent to Solana mempool
- `SettlementComplete` - Confirmed on Solana
- `SettlementFailed` - Temporary failure (will retry)

**Response (Success):**

```json
{
  "success": true,
  "new_version": 2
}
```

**Response (Conflict - Version Mismatch):**

```
HTTP 409 Conflict
{
  "error": "Version mismatch",
  "current_version": 2,
  "expected_version": 1
}
```

**Optimistic Locking:**

- Every update requires `expected_version`
- If current `version != expected_version`, returns HTTP 409
- Prevents duplicate processing across multiple workers
- Successful update increments version: `new_version = expected_version + 1`

#### Casino Statistics

##### GET /api/casino/stats

**Response:**

```json
{
  "total_wagered": 12345.67,
  "total_paid_out": 11234.56,
  "bet_count": 5000,
  "wins": 2400,
  "losses": 2600,
  "wins_24h": 234,
  "losses_24h": 189,
  "wagered_24h": 567.89,
  "paid_out_24h": 623.45
}
```

### Transaction Processor Backend API (Port 3001)

Base URL: `http://localhost:3001`

#### Health

##### GET /health

**Response:**

```json
{
  "status": "ok"
}
```

##### GET /health/detailed

**Response:**

```json
{
  "status": "ok",
  "redis": "connected",
  "postgres": "connected",
  "blockchain_api": "reachable"
}
```

#### Bets

##### POST /api/bets

**Request:**

```json
{
  "user_pubkey": "8x7Y9mK5dQ3pR2nF1vS4wL6eT8hX9oP1aS3fG7jH",
  "game_type": "coinflip",
  "choice": "heads",
  "amount": 1000000000,
  "allowance_pda": "9aB1cD2eF3gH4iJ5kL6mN7oP8qR9sT0uV1wX2yZ3"
}
```

**Response:**

```json
{
  "bet_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "created_at": "2026-02-02T12:34:56Z"
}
```

##### GET /api/bets/:bet_id

**Response:**

```json
{
  "bet_id": "550e8400-...",
  "status": "completed",
  "outcome": "win",
  "payout": 2000000000,
  "solana_tx_id": "5X7Y9mK5dQ...",
  "created_at": "2026-02-02T12:34:56Z",
  "updated_at": "2026-02-02T12:35:12Z"
}
```

##### GET /api/bets?user=:pubkey&limit=20

**Response:**

```json
{
  "bets": [
    {
      "bet_id": "550e8400-...",
      "game_type": "coinflip",
      "amount": 1000000000,
      "status": "completed",
      "created_at": "2026-02-02T12:34:56Z"
    }
  ]
}
```

---

## WebSocket Events

### Connection

**Endpoint:** `ws://localhost:8080/ws`

**Query Parameters:**

- `blocks=true` - Subscribe to new block events
- `casino=true` - Subscribe to casino game events
- `metrics=true` - Subscribe to metrics updates

**Example:**

```javascript
const ws = new WebSocket("ws://localhost:8080/ws?blocks=true&casino=true");
```

### Event Types

#### new_block

```json
{
  "type": "new_block",
  "height": 150000,
  "hash": "a1b2c3d4e5f6...",
  "transactions": ["12345", "12346"],
  "timestamp": 1738483200000
}
```

#### transaction_confirmed

```json
{
  "type": "transaction_confirmed",
  "tx_id": "12345",
  "block_height": 150000,
  "timestamp": 1738483200000
}
```

#### casino_win

```json
{
  "type": "casino_win",
  "game_type": "CoinFlip",
  "wallet": "8x7Y9mK5dQ3pR2nF1vS4wL6eT8hX9oP1aS3fG7jH",
  "amount_won": 2.0,
  "currency": "SOL",
  "tx_id": "12345",
  "block_height": 150000,
  "timestamp": 1738483200000
}
```

#### casino_stats

```json
{
  "type": "casino_stats",
  "total_wagered": 12345.67,
  "total_paid_out": 11234.56,
  "bet_count": 5000,
  "wins": 2400,
  "losses": 2600,
  "timestamp": 1738483200000
}
```

#### metrics

```json
{
  "type": "metrics",
  "tps": 1234.56,
  "pending_transactions": 42,
  "block_height": 150000,
  "timestamp": 1738483200000
}
```

#### heartbeat

```json
{
  "type": "heartbeat",
  "timestamp": 1738483200000
}
```

**Sent every 30 seconds to keep connection alive.**

### Frontend WebSocket Usage

**File:** `transaction-processor/test-ui/src/sdk/websocket/manager.ts`

```typescript
import { WebSocketConnection } from "@/sdk/websocket/manager";

// Create connection
const ws = new WebSocketConnection("ws://localhost:8080/ws", {
  reconnect: true,
  reconnectInterval: 3000,
  maxReconnectAttempts: 10,
});

// Subscribe to events
ws.subscribe<NewBlockEvent>("new_block", (data) => {
  console.log(`New block: ${data.height}`);
});

ws.subscribe<CasinoWinEvent>("casino_win", (data) => {
  toast.success(`${data.wallet} won ${data.amount_won} ${data.currency}!`);
});

// Connect
await ws.connect();

// Disconnect
ws.disconnect();
```

---

## Getting Started

### Prerequisites

1. **Rust** (1.70+)

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (18+) and npm

   ```bash
   brew install node
   ```

3. **Redis**

   ```bash
   brew install redis
   redis-server
   ```

4. **Solana CLI**

   ```bash
   sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
   ```

5. **Anchor CLI** (for smart contract development)
   ```bash
   cargo install --git https://github.com/coral-xyz/anchor --tag v0.30.1 anchor-cli
   ```

### Quick Start

#### 1. Start the Blockchain

```bash
cd backend/blockchain

# Development mode (single validator)
cargo run --release --bin atomiq-unified

# Or with specific options
cargo run --release --bin atomiq-unified -- single-validator \
  --max-tx-per-block 10000 \
  --block-time-ms 1000

# API server only (read-only blockchain explorer)
cargo run --release --bin atomiq-api -- \
  --port 8080 \
  --db-path ./DB/blockchain_data \
  --enable-games true
```

**What it does:**

- Starts DirectCommit consensus engine (1000ms block interval)
- Exposes REST API on port 8080
- Exposes WebSocket on ws://localhost:8080/ws
- Initializes RocksDB at `./DB/blockchain_data`

**Console Output:**

```
[INFO] Starting Atomik Blockchain - Unified Mode
[INFO] Consensus: DirectCommit (1000ms interval)
[INFO] Database: ./DB/blockchain_data
[INFO] API server listening on http://0.0.0.0:8080
[INFO] WebSocket server listening on ws://0.0.0.0:8080/ws
[INFO] Block #0 committed (hash: a1b2c3...)
```

#### 2. Start the Transaction Processor

```bash
cd backend/transaction-processor

# Start all services (backend + processor)
./start.sh
```

**What it does:**

- Starts backend API on port 3001 (bet creation, user management)
- Starts processor service (settlement coordinator + 4 workers)
- Connects to blockchain API at http://localhost:8080
- Connects to Redis for bet queue
- Polls `/api/settlement/pending` every 10 seconds
- Processes settlements on Solana DevNet

**Manual Start (separate terminals):**

```bash
# Terminal 1: Backend
cd services/backend
cargo run --release

# Terminal 2: Processor
cd services/processor
cargo run --release
```

**Stop Services:**

```bash
./stop.sh
```

**Check Logs:**

```bash
tail -f logs/backend.log
tail -f logs/processor.log
```

#### 3. Start Test UI

```bash
cd backend/transaction-processor/test-ui

# Install dependencies (first time only)
npm install

# Start development server
npm run dev
```

**What it does:**

- Starts Vite development server on http://localhost:5173
- Connects to blockchain API at http://localhost:8080
- Connects to WebSocket at ws://localhost:8080/ws
- Provides UI for:
  - Wallet connection (Privy)
  - Vault management (deposit, withdraw)
  - Allowance approval
  - Place bets (coinflip)
  - View game results
  - Live casino feed

**Access the UI:**
Open http://localhost:5173 in your browser

### Full System Test

```bash
# Terminal 1: Blockchain
cd backend/blockchain
cargo run --release --bin atomiq-unified

# Terminal 2: Transaction Processor
cd backend/transaction-processor
./start.sh

# Terminal 3: Test UI
cd backend/transaction-processor/test-ui
npm run dev

# Terminal 4: Check settlement logs
cd backend/transaction-processor
tail -f logs/processor.log | grep settlement
```

### Port Summary

| Service                  | Port | Protocol  | Purpose                              |
| ------------------------ | ---- | --------- | ------------------------------------ |
| **Blockchain API**       | 8080 | HTTP      | REST API for bets, games, settlement |
| **Blockchain WebSocket** | 8080 | WebSocket | Real-time event streaming            |
| **Backend API**          | 3001 | HTTP      | Bet creation, user management        |
| **Test UI**              | 5173 | HTTP      | Frontend development server          |
| **Redis**                | 6379 | TCP       | Bet queue, caching                   |
| **PostgreSQL**           | 5432 | TCP       | User data, bet history               |
| **Prometheus Metrics**   | 9090 | HTTP      | Metrics scraping (optional)          |

---

## Configuration

### Blockchain Configuration

**File:** `blockchain/atomiq.toml`

```toml
[blockchain]
max_transactions_per_block = 10000
max_block_time_ms = 1000

[storage]
data_directory = "./DB/blockchain_data"
write_buffer_size_mb = 256
max_write_buffer_number = 4
compression_type = "Lz4"

[consensus]
mode = "DirectCommit"
direct_commit_interval_ms = 1000

[api]
host = "0.0.0.0"
port = 8080
enable_games = true
tx_queue_capacity = 50000

[websocket]
enabled = true
heartbeat_interval_seconds = 30
```

**CLI Override:**

```bash
cargo run --bin atomiq-api -- \
  --host 0.0.0.0 \
  --port 8080 \
  --db-path ./DB/blockchain_data \
  --enable-games true \
  --tx-queue-capacity 50000
```

### Transaction Processor Configuration

**File:** `transaction-processor/.env`

```bash
# Solana Configuration
SOLANA_NETWORK=devnet
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_COMMITMENT=confirmed
VAULT_PROGRAM_ID=BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ

# Processor Configuration
PROCESSOR_KEYPAIR=./keys/processor-keypair.json
PROCESSOR_WORKER_COUNT=10
PROCESSOR_BATCH_INTERVAL_SECONDS=30

# Settlement Configuration
SETTLEMENT_WORKER_COUNT=4
COORDINATOR_ENABLED=true
COORDINATOR_BATCH_MIN_SIZE=3
COORDINATOR_BATCH_MAX_SIZE=12
COORDINATOR_CHANNEL_BUFFER_SIZE=100

# Blockchain API
BLOCKCHAIN_API_URL=http://localhost:8080
BLOCKCHAIN_API_KEY=settlement-api-key-2026
BLOCKCHAIN_POLL_INTERVAL_SECONDS=10
BLOCKCHAIN_SETTLEMENT_BATCH_SIZE=200

# Redis
REDIS_URL=redis://localhost:6379

# Backend API
API_PORT=3001
API_HOST=0.0.0.0
METRICS_PORT=9090

# Database
DATABASE_URL=postgresql://localhost/atomik_casino
```

**File:** `transaction-processor/services/backend/.env`

```bash
API_PORT=3001
REDIS_URL=redis://localhost:6379
DATABASE_URL=postgresql://localhost/atomik_casino
BLOCKCHAIN_API_URL=http://localhost:8080
```

**File:** `transaction-processor/services/processor/.env`

```bash
SOLANA_RPC_URL=https://api.devnet.solana.com
VAULT_PROGRAM_ID=BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ
PROCESSOR_KEYPAIR=../../keys/processor-keypair.json
BLOCKCHAIN_API_URL=http://localhost:8080
BLOCKCHAIN_API_KEY=settlement-api-key-2026
REDIS_URL=redis://localhost:6379
COORDINATOR_ENABLED=true
SETTLEMENT_WORKER_COUNT=4
```

### Test UI Configuration

**File:** `transaction-processor/test-ui/.env`

```bash
VITE_ATOMIK_API_URL=http://localhost:8080
VITE_WEBSOCKET_ENABLED=true
VITE_SOLANA_RPC_URL=https://api.devnet.solana.com
VITE_VAULT_PROGRAM_ID=BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ
VITE_CASINO_PUBKEY=CasinoKeypair1111111111111111111111111111111
```

### Keypairs Setup

**Generate Keypairs:**

```bash
# Processor keypair (authorized to execute settlements)
solana-keygen new --outfile ./transaction-processor/keys/processor-keypair.json

# Test user keypair
solana-keygen new --outfile ./transaction-processor/keys/test-user-keypair.json

# Casino authority keypair
solana-keygen new --outfile ./transaction-processor/keys/casino-authority-keypair.json
```

**Fund Keypairs (DevNet):**

```bash
# Fund processor (needs SOL for transaction fees)
solana airdrop 2 $(solana-keygen pubkey ./transaction-processor/keys/processor-keypair.json) --url devnet

# Fund test user
solana airdrop 5 $(solana-keygen pubkey ./transaction-processor/keys/test-user-keypair.json) --url devnet
```

**Set Processor in Casino Configuration:**

```bash
# The processor pubkey must match casino.processor in smart contract
solana-keygen pubkey ./transaction-processor/keys/processor-keypair.json
# → Add this to casino initialization script
```

---

## Technical Details

### VRF-Based Provably Fair Gaming

**Algorithm:** Schnorrkel VRF (sr25519)  
**Library:** `schnorrkel = "0.11"`

**How it works:**

1. **Input Creation:**

   ```rust
   let input = format!(
       "tx-{}:{}:{}:{}:block_hash:{},height:{}",
       transaction_id,
       game_type,
       player_address,
       player_choice,
       hex::encode(block_hash),
       block_height
   );
   ```

2. **VRF Signing:**

   ```rust
   let (vrf_output, vrf_proof, _) = keypair.vrf_sign(
       Transcript::new(b"atomik-vrf"),
       &input.as_bytes()
   );
   ```

3. **Outcome Derivation:**

   ```rust
   let first_byte = vrf_output.0[0];
   let coin_result = if first_byte % 2 == 0 {
       CoinFlipResult::Heads
   } else {
       CoinFlipResult::Tails
   };
   ```

4. **Verification:**
   ```rust
   let public_key = keypair.public;
   let is_valid = public_key.vrf_verify(
       Transcript::new(b"atomik-vrf"),
       &input.as_bytes(),
       &vrf_output,
       &vrf_proof
   ).is_ok();
   ```

**Properties:**

- ✅ **Deterministic**: Same input → same output (reproducible)
- ✅ **Unpredictable**: Cannot guess output before generation
- ✅ **Verifiable**: Anyone can verify proof matches output and input
- ✅ **Tamper-proof**: Uses finalized block hash (cannot manipulate after bet)

**VRF Proof Size:** ~96 bytes  
**Generation Time:** 50-100μs  
**Verification Time:** 150-200μs

### Settlement Tracking & Optimistic Locking

**Settlement Status Flow:**

```
PendingSettlement (version: 1)
    ↓ (Worker claims via POST with expected_version: 1)
SubmittedToSolana (version: 2)
    ↓ (Solana TX confirmed)
SettlementComplete (version: 3)
```

**Optimistic Locking Mechanism:**

```rust
// Worker A and Worker B fetch same game (version: 1)

// Worker A: POST /api/settlement/games/12345
//   {status: "SubmittedToSolana", expected_version: 1}
//   → Success! new_version: 2

// Worker B: POST /api/settlement/games/12345
//   {status: "SubmittedToSolana", expected_version: 1}
//   → HTTP 409 Conflict (current version is 2, not 1)
//   → Worker B skips this game

// Worker A: POST /api/settlement/games/12345
//   {status: "SettlementComplete", expected_version: 2, solana_tx_id: "..."}
//   → Success! new_version: 3
```

**Why it works:**

- ✅ Only one worker can claim work (first to update wins)
- ✅ No database locks needed (optimistic)
- ✅ No distributed locks (Redis/Zookeeper) needed
- ✅ Stateless (works across multiple processor instances)

**Retry Logic:**

```rust
// Temporary failure
if error.contains("RPC timeout") {
    status = SettlementFailed;
    retry_count += 1;
    next_retry_after = now + (retry_count * 5_000); // 5s, 10s, 15s
}

// Permanent failure (max retries)
if retry_count >= 3 {
    status = SettlementFailedPermanent;
    // Manual review needed
}
```

**Infinite Retry for Critical Updates:**

After Solana TX succeeds, updating blockchain status is critical:

```rust
async fn update_settlement_complete_with_retry(...) {
    loop {
        match blockchain_api.update_status(...).await {
            Ok(_) => return Ok(()),
            Err(e) if e.contains("409") => return Ok(()), // Already done
            Err(_) => {
                // NEVER GIVE UP - keep retrying with exponential backoff
                sleep(backoff_duration).await;
                backoff_duration = (backoff_duration * 2).min(60);
            }
        }
    }
}
```

**Why infinite retry?**

- If Solana TX succeeded but blockchain update fails, user's funds are moved but status is stale
- Must eventually update blockchain to match reality
- 409 Conflict means another worker succeeded → safe to exit

### Storage Optimizations

**Problem:** Naive storage duplicates data (blocks stored by hash AND height, full TX in indices)

**Solution:** Lightweight indexing with pointers

**Storage Layout:**

| Key Pattern                | Value                     | Size       | Purpose                      |
| -------------------------- | ------------------------- | ---------- | ---------------------------- |
| `block:height:150000`      | Full block data           | ~10KB      | Canonical block storage      |
| `hash_idx:a1b2c3...`       | `150000` (u64)            | 8 bytes    | Hash → height lookup         |
| `tx_idx:12345`             | `150000:0` (height:index) | 16 bytes   | TX → block location          |
| `game:result:tx:12345`     | Full game result          | ~2KB       | Game details with VRF proof  |
| `settlement:pending:12345` | Settlement summary        | ~100 bytes | Lightweight settlement index |

**Savings:**

- **Blocks:** 50% reduction (no duplication by hash)
- **Transactions:** 73% reduction (pointer vs full TX)
- **Settlements:** 95% reduction (summary vs full game)

**Overall:** ~73% storage reduction compared to naive implementation

**RocksDB Configuration:**

```rust
let mut opts = Options::default();
opts.set_compression_type(DBCompressionType::Lz4);
opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB
opts.set_max_write_buffer_number(4);
opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB
opts.create_if_missing(true);
```

### DirectCommit Consensus

**Mode:** Single-validator DirectCommit (production)  
**Alternative:** HotStuff BFT consensus (multi-validator, not used in production)

**DirectCommit Algorithm:**

```rust
loop {
    // 1. Wait for interval (1000ms)
    sleep(Duration::from_millis(1000)).await;

    // 2. Drain transaction pool (max 10,000 TXs)
    let transactions = tx_pool.drain(10_000);

    if transactions.is_empty() {
        continue; // Skip empty blocks
    }

    // 3. Execute transactions (deterministic state updates)
    let state_updates = execute_transactions(&transactions);

    // 4. Create block
    let block = Block {
        height: current_height + 1,
        previous_block_hash: latest_block_hash,
        transactions,
        transactions_root: merkle_root(&transactions),
        state_root: hash(&state_updates),
        timestamp: now(),
        block_hash: calculate_block_hash(...),
    };

    // 5. Commit to RocksDB (atomic)
    storage.store_block(&block)?;

    // 6. Broadcast finalization event
    websocket_manager.broadcast_new_block(block.height, block.hash, tx_ids).await;

    // 7. For game transactions: Generate VRF outcomes AFTER finalization
    for tx in game_transactions {
        let vrf_result = vrf_engine.generate_outcome(
            tx.id,
            tx.game_type,
            tx.player_choice,
            block.hash, // ← Uses finalized block hash (tamper-proof)
            block.height
        );

        game_store.save_result(tx.id, vrf_result)?;
    }

    current_height += 1;
}
```

**Why DirectCommit for Production:**

- ✅ **Fast:** 10-20ms finality (no consensus overhead)
- ✅ **Simple:** No leader election, no view changes
- ✅ **Reliable:** No network partitions (single validator)
- ✅ **Deterministic:** Perfect for provably fair gaming

**Trade-off:** Single point of failure (acceptable for DevNet casino)

**Future:** Can migrate to HotStuff BFT for decentralization (3+ validators)

### Coordinator-Worker Architecture

**Problem:** Multiple workers polling same API causes race conditions

**Solution:** Central coordinator distributes work via channels

**Architecture:**

```
Coordinator (single thread)
    ↓ GET /api/settlement/pending?limit=200
    ↓ Group by outcome: [Wins: [...], Losses: [...]]
    ↓ Create batches (3-12 settlements per batch)
    ↓ Round-robin distribution
    ├────────┬────────┬────────┐
    ↓        ↓        ↓        ↓
Worker 1  Worker 2  Worker 3  Worker 4
(channel) (channel) (channel) (channel)
```

**Coordinator Logic:**

```rust
loop {
    // 1. Fetch pending settlements
    let games = blockchain_api.get_pending_settlements(200).await?;

    // 2. Group by outcome
    let wins: Vec<_> = games.iter().filter(|g| g.outcome == "Win").collect();
    let losses: Vec<_> = games.iter().filter(|g| g.outcome == "Loss").collect();

    // 3. Create batches
    let batches = create_batches(&wins, &losses, min: 3, max: 12);

    // 4. Distribute to workers (round-robin)
    for (i, batch) in batches.iter().enumerate() {
        let worker_idx = i % num_workers;
        worker_channels[worker_idx].send(batch).await?;
    }

    // 5. Wait before next poll (10 seconds)
    sleep(Duration::from_secs(10)).await;
}
```

**Worker Logic:**

```rust
loop {
    // Wait for batch from coordinator
    let batch = channel.recv().await?;

    // Process each settlement in batch
    for game in batch.settlements {
        match process_settlement(game).await {
            Ok(_) => metrics.increment("settlements_success"),
            Err(e) => {
                log::error!("Settlement failed: {}", e);
                metrics.increment("settlements_failed");
            }
        }
    }
}
```

**Benefits:**

- ✅ No race conditions (coordinator owns fetching)
- ✅ Load balancing (round-robin distribution)
- ✅ Parallel processing (4 workers process simultaneously)
- ✅ Batching (related settlements grouped together)

---

## Current Status

### ✅ Fully Implemented & Working

**Blockchain:**

- ✅ DirectCommit consensus (10-20ms finality)
- ✅ VRF-based game processing (provably fair)
- ✅ RocksDB storage with 73% optimization
- ✅ Settlement API with optimistic locking
- ✅ WebSocket real-time updates
- ✅ Finalization event system
- ✅ CoinFlip game fully functional
- ✅ Transaction verification endpoints
- ✅ Block explorer API

**Smart Contracts:**

- ✅ Vault program deployed to DevNet (`BtZT2B1...`)
- ✅ Allowance-based gasless betting
- ✅ Direct lamports manipulation (~100 CU vs ~5,000 CU)
- ✅ Deduplication via `ProcessedBet` accounts
- ✅ Native SOL support
- ✅ Admin controls (pause, withdraw profits)

**Transaction Processor:**

- ✅ Coordinator + worker architecture
- ✅ Parallel settlement processing (4 workers)
- ✅ Optimistic locking for race prevention
- ✅ Retry logic with exponential backoff (max 3 attempts)
- ✅ Infinite retry for critical updates
- ✅ Real Solana transactions to DevNet
- ✅ Circuit breaker for RPC failures

**Frontend:**

- ✅ React + Vite test UI
- ✅ Solana wallet integration (Privy)
- ✅ WebSocket live updates
- ✅ API client with retry logic
- ✅ Vault management components
- ✅ Betting interface

### 🚧 In Progress / Limited Implementation

- 🚧 **Multi-Game Support:** Only CoinFlip implemented (Dice, Plinko, Crash planned)
- 🚧 **SPL Token Support:** Infrastructure exists, only SOL fully tested
- 🚧 **Batched Settlements:** Processes 1-12 settlements per worker (can optimize to 50+ per Solana TX)
- 🚧 **Admin Dashboard:** CLI tools exist, no web UI
- 🚧 **Monitoring:** Prometheus metrics exposed, no Grafana dashboards
- 🚧 **Load Testing:** Tested up to 10K TPS, not 100K

### ⚠️ Known Limitations

- ⚠️ **DevNet Only:** Not audited or deployed to MainNet
- ⚠️ **Single Validator:** DirectCommit mode (no decentralization)
- ⚠️ **No Multi-Region:** Single-server deployment
- ⚠️ **Limited Error Recovery:** Some edge cases need manual intervention
- ⚠️ **No Rate Limiting:** API endpoints unprotected (DoS vulnerable)

### 🎯 Next Steps

**Phase 1: Production Hardening (1-2 weeks)**

- [ ] Add rate limiting to API endpoints
- [ ] Implement Grafana dashboards for monitoring
- [ ] Add alerting (queue depth, failure rates, RPC errors)
- [ ] Database backup automation
- [ ] Load testing (sustained 1000+ RPS)
- [ ] Security audit (smart contracts)

**Phase 2: Feature Expansion (2-4 weeks)**

- [ ] Implement Dice game
- [ ] Implement Plinko game
- [ ] Add SPL token support (USDC)
- [ ] Build admin web dashboard
- [ ] User profile system

**Phase 3: Optimization (2-3 weeks)**

- [ ] Batch settlements (50+ per Solana TX)
- [ ] Implement outbox pattern for >100 RPS
- [ ] Add settlement leasing (distributed workers)
- [ ] Optimize RocksDB for high write throughput

**Phase 4: MainNet Preparation (4-6 weeks)**

- [ ] Smart contract audit by third party
- [ ] Fuzz testing
- [ ] Penetration testing
- [ ] HSM/KMS for key management
- [ ] Multi-region deployment
- [ ] Comprehensive monitoring & alerting
- [ ] Disaster recovery procedures

---

## Development Guide

### Running Tests

**Blockchain Tests:**

```bash
cd blockchain

# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Specific test
cargo test test_vrf_engine

# With output
cargo test -- --nocapture
```

**Transaction Processor Tests:**

```bash
cd transaction-processor

# Backend tests
cd services/backend
cargo test

# Processor tests
cd services/processor
cargo test

# Shared module tests
cd services/shared
cargo test
```

**Smart Contract Tests:**

```bash
cd blockchain/solana-playground-deploy

# Run all tests
anchor test

# Specific test
anchor test --skip-build -- --test test_approve_allowance

# Local validator
anchor test --detach
```

**Frontend Tests:**

```bash
cd transaction-processor/test-ui

npm test
npm run test:coverage
```

### Utility Scripts

**Blockchain Scripts:**

```bash
cd blockchain/scripts

# Test all APIs
./test_api.sh

# Test games
./test_games.sh

# Test settlement API
./test_settlement.sh

# Test restart stability
./test_restart_stability.sh
```

**Transaction Processor Scripts:**

```bash
cd transaction-processor/scripts

# View logs
./view-logs.sh

# Check casino vault balance
node check-casino-vault.js

# Initialize vaults
./initialize-vaults.sh

# Place real bet (DevNet)
node place-real-bet.js
```

### Troubleshooting

**Blockchain won't start:**

```bash
# Check if RocksDB is locked
rm -rf blockchain/DB/blockchain_data/LOCK

# Check if port 8080 is in use
lsof -i :8080
kill -9 <PID>

# Clear all data and restart
rm -rf blockchain/DB/blockchain_data
cargo run --release --bin atomiq-unified
```

**Settlement processor stuck:**

```bash
# Check processor logs
tail -f transaction-processor/logs/processor.log

# Check pending settlements
curl http://localhost:8080/api/settlement/pending?limit=10 \
  -H "X-API-Key: settlement-api-key-2026"

# Restart processor
cd transaction-processor
./stop.sh
./start.sh
```

**Solana transactions failing:**

```bash
# Check RPC health
curl https://api.devnet.solana.com -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'

# Check processor balance (needs SOL for fees)
solana balance $(solana-keygen pubkey ./keys/processor-keypair.json) --url devnet

# Airdrop if needed
solana airdrop 2 $(solana-keygen pubkey ./keys/processor-keypair.json) --url devnet
```

**WebSocket disconnecting:**

```bash
# Check WebSocket connection
wscat -c ws://localhost:8080/ws?blocks=true

# Check nginx/proxy timeouts (if using)
# Increase timeout to 60+ seconds
```

### Database Management

**RocksDB (Blockchain):**

```bash
# Inspect database
cargo run --bin inspect-db-main -- --db-path ./DB/blockchain_data

# Backup database
tar -czf blockchain_backup_$(date +%Y%m%d).tar.gz blockchain/DB/blockchain_data/

# Restore database
tar -xzf blockchain_backup_20260202.tar.gz -C blockchain/DB/
```

**PostgreSQL (Transaction Processor):**

```bash
# Create database
createdb atomik_casino

# Run migrations (if using sqlx/diesel)
cd transaction-processor/services/backend
cargo sqlx migrate run

# Backup
pg_dump atomik_casino > backup_$(date +%Y%m%d).sql

# Restore
psql atomik_casino < backup_20260202.sql
```

**Redis:**

```bash
# Check keys
redis-cli KEYS "bet:*"

# Clear all bets
redis-cli DEL $(redis-cli KEYS "bet:*")

# Monitor commands
redis-cli MONITOR
```

---

## Security Considerations

### Smart Contract Security

✅ **Implemented:**

- Deduplication via `ProcessedBet` accounts (prevents replay attacks)
- Allowance expiry (max 24 hours)
- Amount limits (min 0.01 SOL, max 1000 SOL per bet, max 10,000 SOL allowance)
- Rate limiting (100 allowances/hour per user)
- Processor authorization (only `casino.processor` can execute settlements)
- Pausing mechanism (emergency stop)
- Balance reconciliation (tracks `sol_balance` vs actual lamports)

⚠️ **TODO:**

- [ ] Third-party audit
- [ ] Fuzz testing (Trident)
- [ ] Formal verification of critical paths

### API Security

⚠️ **Current State:**

- Settlement API uses static API key (`X-API-Key` header)
- No rate limiting on public endpoints
- No authentication on game endpoints

🎯 **Recommended:**

- [ ] Add rate limiting (100 req/min per IP)
- [ ] Implement JWT authentication for user-specific endpoints
- [ ] Rotate API keys regularly
- [ ] Add CORS restrictions in production
- [ ] Enable HTTPS with proper certificates

### Keypair Management

⚠️ **Current State:**

- Keypairs stored as JSON files
- No encryption at rest
- Loaded into memory at startup

🎯 **Production Requirements:**

- [ ] Use HSM (Hardware Security Module) or KMS (Key Management Service)
- [ ] Implement key rotation
- [ ] Encrypt keypairs at rest
- [ ] Audit all key access
- [ ] Separate keys by environment (dev/staging/prod)

---

## Architecture Diagrams

### System Component Interaction

```
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND LAYER                          │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         React UI (transaction-processor/test-ui)         │  │
│  │                                                          │  │
│  │  • Wallet integration (Privy)                            │  │
│  │  • Betting interface                                     │  │
│  │  • Vault management                                      │  │
│  │  • WebSocket live updates                                │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ ↑
          HTTP/WebSocket    ↓ ↑    Direct Solana TX
                            ↓ ↑
┌──────────────────────────────────┬──────────────────────────────┐
│                                  │                              │
│    BLOCKCHAIN LAYER              │    SOLANA LAYER              │
│    (blockchain/)                 │    (Smart Contracts)         │
│                                  │                              │
│  ┌────────────────────────────┐ │ ┌──────────────────────────┐ │
│  │   API Server (Axum)        │ │ │   Vault Program          │ │
│  │   Port 8080                │ │ │   BtZT2B1NkEG...         │ │
│  │                            │ │ │                          │ │
│  │  • Game endpoints          │ │ │  • User vaults (PDAs)    │ │
│  │  • Settlement API          │ │ │  • Allowances            │ │
│  │  • WebSocket events        │ │ │  • Casino vault          │ │
│  └────────────────────────────┘ │ │  • Deduplication         │ │
│                ↓                 │ └──────────────────────────┘ │
│  ┌────────────────────────────┐ │              ↑               │
│  │   DirectCommit Engine      │ │              │               │
│  │   (Consensus)              │ │              │               │
│  │                            │ │              │               │
│  │  • 1000ms block interval   │ │              │               │
│  │  • VRF game processing     │ │              │               │
│  │  • Transaction execution   │ │              │               │
│  └────────────────────────────┘ │              │               │
│                ↓                 │              │               │
│  ┌────────────────────────────┐ │              │               │
│  │   RocksDB Storage          │ │              │               │
│  │                            │ │              │               │
│  │  • Blocks                  │ │              │               │
│  │  • Game results            │ │              │               │
│  │  • Settlement index        │ │              │               │
│  └────────────────────────────┘ │              │               │
│                ↑                 │              │               │
└────────────────┼─────────────────┴──────────────┼───────────────┘
                 │                                │
                 │    HTTP Settlement API         │ Solana RPC
                 │                                │
         ┌───────┴────────────────────────────────┴───────┐
         │                                                │
         │    TRANSACTION PROCESSOR LAYER                 │
         │    (transaction-processor/)                    │
         │                                                │
         │  ┌──────────────────────────────────────────┐ │
         │  │   Settlement Coordinator                 │ │
         │  │                                          │ │
         │  │  • Polls blockchain every 10s            │ │
         │  │  • Fetches pending settlements           │ │
         │  │  • Creates batches (3-12 per batch)      │ │
         │  │  • Distributes to workers                │ │
         │  └──────────────────────────────────────────┘ │
         │             ↓        ↓        ↓               │
         │    ┌────────┴────┬───┴────┬───┴────────┐     │
         │    │             │        │            │     │
         │  ┌─▼──┐       ┌──▼─┐   ┌──▼─┐      ┌──▼─┐   │
         │  │ W1 │       │ W2 │   │ W3 │      │ W4 │   │
         │  └────┘       └────┘   └────┘      └────┘   │
         │                                                │
         │  • Build Solana TX                             │
         │  • Submit to Solana RPC                        │
         │  • Update settlement status                    │
         │  • Optimistic locking                          │
         └────────────────────────────────────────────────┘
```

---

## Contributing

### Code Style

**Rust:**

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Fix warnings
cargo clippy --fix
```

**TypeScript:**

```bash
# Format code
npm run format

# Lint
npm run lint
```

### Commit Guidelines

```
feat: Add dice game implementation
fix: Resolve settlement race condition
docs: Update API documentation
test: Add VRF verification tests
refactor: Simplify coordinator logic
```

### Pull Request Process

1. Fork the repository
2. Create feature branch: `git checkout -b feature/dice-game`
3. Make changes with tests
4. Run all tests: `cargo test --all`
5. Commit with descriptive message
6. Push to fork: `git push origin feature/dice-game`
7. Open pull request with detailed description

---

## License

[Add license information]

---

## Support

**Issues:** [GitHub Issues]  
**Documentation:** This README + `/blockchain/docs/` + `/transaction-processor/docs/`  
**Contact:** [Add contact information]

---

## Acknowledgments

- **HotStuff-rs** - BFT consensus framework
- **Schnorrkel** - Polkadot VRF implementation
- **Anchor** - Solana smart contract framework
- **RocksDB** - High-performance key-value store

---

**Last Updated:** February 2, 2026  
**Version:** 1.0.0-devnet
