# Atomik Wallet - Solana Vault POC

Non-custodial Solana wallet system with vault-based allowances and batched bet settlement.

## Architecture

```
atomik-wallet/
├── programs/           # Solana programs (Anchor)
│   └── vault/         # Vault program with allowance system
├── services/          # Backend services (Rust)
│   ├── backend/       # API server (Axum)
│   └── processor/     # External processor for batch settlement
├── apps/              # Frontend applications
│   └── frontend/      # React + Next.js + Privy
└── packages/          # Shared packages
    ├── types/         # TypeScript types
    └── domain/        # Domain models
```

## Features

- ✅ Non-custodial vault system with PDA-based accounts
- ✅ One-time allowance approval (no per-bet signing)
- ✅ Batched settlement with parallel processing
- ✅ Comprehensive error handling and reconciliation
- ✅ High-security design against common Solana vulnerabilities
- ✅ Production-ready architecture with DDD principles

## Getting Started

### Prerequisites

- Node.js >= 18
- pnpm >= 8
- Rust >= 1.75
- Anchor >= 0.29
- Solana CLI >= 1.17
- PostgreSQL >= 15
- Redis >= 7

### Installation

```bash
# Install dependencies
pnpm install

# Build all projects
pnpm build

# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Run database migrations
pnpm db:migrate
```

### Development

```bash
# Start all services in dev mode
pnpm dev

# Or start individually:
pnpm anchor:build        # Build Solana program
pnpm backend:dev         # Start API server
pnpm processor:dev       # Start batch processor
pnpm frontend:dev        # Start frontend
```

### Testing

```bash
# Run all tests
pnpm test

# Test Anchor program
pnpm anchor:test

# Run integration tests
cd services/backend && cargo test
```

### Deployment

```bash
# Deploy to Solana testnet
pnpm anchor:deploy:testnet

# Build production
pnpm build
```

## Security

This system handles real money. Security considerations:

- ✅ All arithmetic uses checked operations
- ✅ Signer validation on all privileged operations
- ✅ PDA canonical bump storage
- ✅ SPL token account validation
- ✅ Allowance rate limiting and expiry enforcement
- ✅ Idempotency keys on all financial operations
- ✅ Immutable audit log
- ✅ Circuit breakers on RPC calls
- ✅ Transaction verification before signing

See [SECURITY.md](SECURITY.md) for full security documentation.

## License

MIT
# atomiq-bet-settlement
