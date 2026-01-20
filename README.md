# Atomik Wallet - Solana Betting System

Non-custodial Solana betting platform with vault-based allowances and batched transaction processing.

## Architecture

```
atomik-wallet/
├── solana-playground-deploy/  # Solana program (Anchor)
│   └── programs/vault/        # Vault program with allowances
├── services/                  # Backend services (Rust)
│   ├── backend/              # REST API (Actix-web)
│   └── processor/            # Batch processor
├── test-ui/                  # Test interface (React + Vite)
├── scripts/                  # Deployment & testing scripts
├── docs/                     # Documentation
└── keys/                     # Private keys (gitignored)
```

## Technical Stack

**Blockchain:** Solana Devnet  
**Smart Contract:** Anchor 0.30.1  
**Backend:** Rust (Actix-web, Tokio)  
**Processor:** Rust (Tokio, multi-threaded workers)  
**Storage:** Redis (in-memory bet management)  
**Frontend:** React 18 + Vite + Solana wallet adapters

## Key Features

- **Program-owned Casino Vault** - Direct lamports manipulation (~100 CU vs ~5,000 CU via CPI)
- **Allowance System** - One-time approval for multiple bets (no per-bet signing)
- **Batched Settlement** - Parallel processing with worker pool
- **Race Condition Prevention** - Frontend validates allowance before bet submission
- **Transaction Deduplication** - Unique memo instructions prevent duplicate processing
- **Balance Reconciliation** - Admin tool to sync tracked vs actual balances

## Quick Start

### Prerequisites

- Node.js 18+
- pnpm 8+
- Rust 1.75+
- Solana CLI 1.17+
- Redis 7+

### Setup

```bash
# Install dependencies
brew install redis

# Start system (single command)
./start.sh

# Start test UI (optional)
cd test-ui && pnpm dev
```

### Stop System

```bash
./stop.sh
```

### Deploy to Solana Playground

1. Open [Solana Playground](https://beta.solpg.io)
2. Import files from `solana-playground-deploy/programs/vault/`
3. Run `build` and `deploy`
4. Update `.env` files with new program ID

## Core Concepts

### Casino Vault (Program-Owned)

```rust
// seeds: [b"casino-vault", casino.key()]
pub struct CasinoVault {
    pub casino: Pubkey,
    pub sol_balance: u64,  // Tracked balance
    pub bump: u8,
}
```

Direct lamports manipulation:

```rust
**casino_vault.to_account_info().try_borrow_mut_lamports()? -= amount;
```

### Allowance Flow

1. User approves allowance (e.g., 5 SOL for 10,000 seconds)
2. Frontend validates allowance exists on-chain
3. User places bets without signing each one
4. Processor spends from allowance via `spend_from_allowance` instruction

### Bet Processing Pipeline

```
User → Backend API → Redis Queue → Processor Workers → Solana → Payout
       (creates bet)  (batching)   (parallel exec)   (confirm)  (winner)
```

## Security

- All privileged operations require casino authority signature
- Allowance constraints: max duration (24h), max amount (10,000 SOL)
- Rate limiting on allowance approvals (100/hour)
- Bet deduplication via `ProcessedBet` accounts
- Account mutability validation (even for placeholders)

## Scripts

See `scripts/` directory:

- `start-services.sh` - Start backend + processor
- `stop-services.sh` - Stop all services
- `initialize-casino-vault.js` - Initialize casino (one-time)
- `fund-casino-vault.js` - Fund casino vault
- `test-real-bet.sh` - End-to-end bet test

## Documentation

See `docs/` directory for detailed documentation:

- `CASINO_VAULT_IMPLEMENTATION.md` - Architecture details
- `DEPLOYMENT_2026-01-17.md` - Latest deployment notes
- `SOLANA_PLAYGROUND_GUIDE.md` - Deployment guide

## License

MIT
