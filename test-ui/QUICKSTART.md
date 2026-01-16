# 🚀 Quick Start Guide

## Get Started in 3 Steps

### 1. Setup

```bash
cd test-ui
chmod +x setup.sh
./setup.sh
```

### 2. Configure Privy

1. Go to [console.privy.io](https://console.privy.io)
2. Create a new app or use existing
3. Enable **Solana** in the dashboard
4. Copy your **App ID**
5. Update `test-ui/.env`:
   ```env
   VITE_PRIVY_APP_ID=clxxxxxxxxxxxxxx
   ```

### 3. Start

```bash
pnpm dev
```

Visit [http://localhost:3000](http://localhost:3000)

---

## Testing Flow

1. **Connect Wallet** → Click "Connect with Privy"
2. **Get Funds** → Click "Request Airdrop" (1 SOL devnet)
3. **Place Bet** → Enter amount, choose heads/tails
4. **Watch Results** → Monitor transaction log
5. **Verify** → Click TX link to view on Solana Explorer

---

## Requirements

✅ Backend running on `localhost:3001`  
✅ Processor service running  
✅ `USE_REAL_SOLANA=true` in backend `.env`  
✅ Privy App ID configured  

---

## Troubleshooting

**Backend not responding?**
```bash
cd ../
curl http://localhost:3001/health
```

**No transactions appearing?**
- Check if processor is running
- Verify `USE_REAL_SOLANA=true` in backend .env
- Check processor logs for errors

**Airdrop failing?**
- Devnet faucet rate limits exist
- Use [faucet.solana.com](https://faucet.solana.com) as backup

---

## What You'll See

### Transaction Statuses

- **Pending** → Created, waiting for processor
- **Batched** → In batch, building transaction  
- **SubmittedToSolana** → Sent to blockchain
- **ConfirmedOnSolana** → On-chain confirmation
- **Completed** → Fully settled with results

### Real Transaction IDs

Unlike simulation mode, you'll see **real Solana transaction signatures**:
- Format: `4kdrThKmcBH...` (base58)
- Viewable on Solana Explorer
- Confirmed on devnet blockchain

---

## Architecture

```
┌─────────────┐
│  Test UI    │ ← You are here
│  (React)    │
└──────┬──────┘
       │
       ↓ HTTP
┌──────────────┐
│  Backend API │ ← localhost:3001
│  (Rust/Axum) │
└──────┬───────┘
       │
       ↓ Database
┌──────────────┐      ┌────────────┐
│  Processor   │ ←───→│   Solana   │
│  (Rust)      │      │   Devnet   │
└──────────────┘      └────────────┘
```

---

## Support

Questions? Check the full [README.md](./README.md) for detailed documentation.