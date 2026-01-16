# Atomik Bet Test UI

A React-based testing interface for the Atomik Wallet betting system on Solana Devnet.

## Features

- ✅ **Privy Wallet Integration** - Easy wallet connection via email, SMS, or wallet
- ✅ **Solana Devnet Support** - Test with real transactions on devnet
- ✅ **Vault Management** - Automatic PDA derivation for user vaults
- ✅ **Betting Interface** - Place coinflip bets with real SOL
- ✅ **Transaction Tracking** - Real-time monitoring of bet status
- ✅ **Explorer Integration** - Direct links to Solana Explorer for verification
- ✅ **Error Logging** - Comprehensive error tracking and display

## Prerequisites

- Node.js >= 18
- pnpm (recommended) or npm
- A Privy account and App ID from [console.privy.io](https://console.privy.io)
- Backend API running on `localhost:3001` (or configured endpoint)

## Setup

### 1. Install Dependencies

```bash
pnpm install
# or
npm install
```

### 2. Configure Environment

Create a `.env` file in the `test-ui` directory:

```env
VITE_PRIVY_APP_ID=your_privy_app_id_here
VITE_API_BASE_URL=http://localhost:3001
VITE_SOLANA_RPC_URL=https://api.devnet.solana.com
VITE_VAULT_PROGRAM_ID=HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4
VITE_SOLANA_NETWORK=devnet
```

### 3. Get a Privy App ID

1. Go to [console.privy.io](https://console.privy.io)
2. Create a new app
3. Enable Solana support in the dashboard
4. Copy your App ID to the `.env` file

### 4. Start the Development Server

```bash
pnpm dev
# or
npm run dev
```

The app will be available at `http://localhost:3000`

## Usage Guide

### 1. Connect Wallet
- Click "Connect with Privy"
- Choose connection method (email, SMS, or existing wallet)
- Privy will create a Solana wallet for you if you don't have one

### 2. Get Devnet SOL
- Click "Request Airdrop" button to get 1 SOL on devnet
- Wait for confirmation (usually a few seconds)
- Your balance will update automatically

### 3. Check Vault
- Your vault address is automatically derived from your wallet
- This is a PDA (Program Derived Address) specific to your wallet

### 4. Place a Bet
- Enter amount (minimum 0.1 SOL)
- Choose heads or tails
- Click "Place Bet"
- Watch the transaction log for status updates

### 5. Monitor Results
- Transaction log updates every 5 seconds
- Click transaction IDs to view on Solana Explorer
- See win/loss status and payouts
- View any errors or retry attempts

## How It Works

### Transaction Flow

1. **Bet Creation** → API creates bet record in database (Status: Pending)
2. **Batch Processing** → Processor picks up pending bets (Status: Batched)
3. **Solana Submission** → Transaction sent to devnet (Status: SubmittedToSolana)
4. **Confirmation** → Blockchain confirms transaction (Status: ConfirmedOnSolana)
5. **Completion** → Bet settled, results processed (Status: Completed)

### Status Meanings

- **Pending**: Bet created, waiting for processor
- **Batched**: Included in a batch, preparing transaction
- **SubmittedToSolana**: Transaction sent to blockchain
- **ConfirmedOnSolana**: Transaction confirmed on-chain
- **Completed**: Bet fully settled, results finalized
- **FailedRetryable**: Temporary failure, will retry
- **FailedManualReview**: Permanent failure, needs review

## Architecture

```
test-ui/
├── src/
│   ├── components/       # React components
│   │   ├── WalletConnect.tsx      # Privy wallet integration
│   │   ├── VaultManager.tsx       # Vault PDA management
│   │   ├── BettingInterface.tsx   # Bet placement UI
│   │   └── TransactionLog.tsx     # Transaction history
│   ├── hooks/           # Custom React hooks
│   │   ├── useApi.ts             # Backend API integration
│   │   └── useTransactions.ts    # Transaction polling
│   ├── services/        # External services
│   │   ├── api.ts               # API client
│   │   └── solana.ts            # Solana web3 utilities
│   ├── types/          # TypeScript types
│   └── App.tsx         # Main application
```

## API Integration

The UI communicates with the backend API:

### Endpoints Used

- `POST /api/bets` - Create new bet
- `GET /api/external/bets/pending` - Get all bets
- `GET /health` - Backend health check

## Troubleshooting

### Airdrop Fails
- Devnet faucet has rate limits
- Try using the official faucet at [faucet.solana.com](https://faucet.solana.com)
- Wait a few minutes between requests

### Bet Not Processing
- Check if backend API is running on `localhost:3001`
- Verify processor service is running
- Check browser console for errors
- Ensure `USE_REAL_SOLANA=true` in backend .env

### Transaction Not Appearing
- Backend may be in simulation mode
- Check processor logs for errors
- Verify Program ID matches deployed contract

## Development

### Build for Production

```bash
pnpm build
# or
npm run build
```

### Preview Production Build

```bash
pnpm preview
# or
npm run preview
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VITE_PRIVY_APP_ID` | Your Privy App ID | Required |
| `VITE_API_BASE_URL` | Backend API URL | `http://localhost:3001` |
| `VITE_SOLANA_RPC_URL` | Solana RPC endpoint | `https://api.devnet.solana.com` |
| `VITE_VAULT_PROGRAM_ID` | Deployed program ID | See .env |
| `VITE_SOLANA_NETWORK` | Network cluster | `devnet` |

## Tech Stack

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Privy** - Wallet authentication
- **@solana/web3.js** - Solana blockchain interaction
- **Tailwind CSS** - Styling
- **Lucide React** - Icons

## License

MIT