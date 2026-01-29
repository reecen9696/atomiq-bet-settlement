# 🚀 Reusable Solana React SDK - Quick Start Guide

## Overview

This guide shows you how to extract and use the reusable components from the Atomik casino project for your own Solana-based React application.

## 📦 What You Get

### ✅ **SDK Package** (`src/sdk/`)

- **Configuration System**: Generic, blockchain-agnostic configuration
- **API Client**: REST client with retry logic and error handling
- **Vault Operations**: Account management, deposits, withdrawals
- **Transaction Utils**: PDA derivation, memo instructions, error handling
- **WebSocket Manager**: Real-time connection management with auto-reconnect

### ✅ **UI Components** (`src/components/`)

- **ReusableWalletConnect**: Wallet connection with balance display and airdrop
- **Transaction Utilities**: Common transaction patterns and helpers

### ✅ **Utilities** (`src/utils/`)

- **Transaction Builders**: Chainable transaction construction
- **Error Handling**: Typed error classes for better debugging
- **Format Helpers**: SOL/lamports conversion, explorer URLs

## 🏗️ Setting Up a New Project

### 1. Copy Required Files

Copy these files/folders to your new React project:

```bash
# Core SDK
src/sdk/
  ├── env.ts                     # Configuration system
  ├── index.ts                   # Main exports
  ├── api/client.ts             # REST API client
  ├── utils/memo.ts             # Memo instructions
  ├── websocket/manager.ts      # WebSocket connections
  └── solana/                   # Solana-specific services
      ├── vault.ts
      ├── allowance.ts
      └── betting.ts

# UI Components
src/components/
  └── ReusableWalletConnect.tsx

# Utilities
src/utils/
  └── transactions.ts
```

### 2. Install Dependencies

```bash
npm install @solana/web3.js @solana/wallet-adapter-base @solana/wallet-adapter-react @solana/wallet-adapter-react-ui @solana/wallet-adapter-wallets
```

### 3. Basic App Setup

```tsx
// App.tsx
import { useMemo } from "react";
import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import { clusterApiUrl } from "@solana/web3.js";
import { createGenericConfig, createAtomikSDK } from "./sdk";
import { ReusableWalletConnect } from "./components/ReusableWalletConnect";

// Import wallet adapter CSS
import "@solana/wallet-adapter-react-ui/styles.css";

function App() {
  const network = WalletAdapterNetwork.Devnet; // or Mainnet
  const endpoint = useMemo(() => clusterApiUrl(network), [network]);

  const wallets = useMemo(
    () => [new PhantomWalletAdapter(), new SolflareWalletAdapter()],
    [],
  );

  // Configure your SDK
  const config = createGenericConfig({
    blockchain: {
      network: "devnet", // or 'mainnet'
      programId: "YOUR_PROGRAM_ID_HERE",
      rpcUrl: endpoint,
      commitment: "confirmed",
      confirmTimeout: 30000,
    },
    api: {
      baseUrl: "https://your-api.com",
      apiKey: "your-api-key",
      timeout: 10000,
      retryAttempts: 3,
    },
    websocket: {
      enabled: true,
      reconnectAttempts: 5,
      reconnectDelay: 1000,
      connectionTimeout: 10000,
    },
  });

  const sdk = createAtomikSDK(config);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <div className="App">
            <header className="App-header">
              <h1>My Solana App</h1>

              {/* Reusable wallet component */}
              <ReusableWalletConnect
                config={config}
                showBalance={true}
                showAirdrop={true}
                showExplorer={true}
                onWalletConnect={(publicKey) =>
                  console.log("Connected:", publicKey)
                }
                onBalanceUpdate={(balance) => console.log("Balance:", balance)}
              />

              {/* Your app content */}
            </header>
          </div>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}

export default App;
```

## 🔧 Configuration Options

### Generic Configuration (Recommended)

```typescript
import { createGenericConfig, createAtomikSDK } from "./sdk";

const config = createGenericConfig({
  blockchain: {
    network: "mainnet", // 'devnet' | 'testnet' | 'mainnet'
    programId: "YOUR_PROGRAM_ID",
    rpcUrl: "https://api.mainnet-beta.solana.com",
    commitment: "confirmed", // 'processed' | 'confirmed' | 'finalized'
    confirmTimeout: 30000, // Transaction confirmation timeout
  },
  api: {
    baseUrl: "https://your-backend-api.com",
    apiKey: "your-api-key", // Optional
    timeout: 10000,
    retryAttempts: 3,
  },
  websocket: {
    enabled: true,
    reconnectAttempts: 5,
    reconnectDelay: 1000,
    connectionTimeout: 10000,
  },
});

const sdk = createAtomikSDK(config);
```

### Environment-Based Configuration

```typescript
import { createConfigFromParams } from "./sdk";

const config = createConfigFromParams({
  apiBaseUrl: process.env.REACT_APP_API_URL,
  apiKey: process.env.REACT_APP_API_KEY,
  blockchainNetwork: process.env.REACT_APP_NETWORK,
  rpcUrl: process.env.REACT_APP_RPC_URL,
  programId: process.env.REACT_APP_PROGRAM_ID,
});
```

## 💼 Using SDK Services

### API Client

```typescript
import { createApiClient } from "./sdk";

const apiClient = createApiClient(config);

// Make API calls
const response = await apiClient.get("/endpoint");
const postResult = await apiClient.post("/submit", { data: "value" });
```

### Vault Operations

```typescript
import { createVaultService } from "./sdk";

const vaultService = createVaultService(config);

// Derive vault PDA
const vaultPda = await vaultService.deriveVaultPDA(userPublicKey);

// Check vault exists
const vaultInfo = await vaultService.getVaultInfo(userPublicKey);

// Initialize vault
const initTx = await vaultService.initializeVault(userPublicKey, 1.0);
```

### Transaction Utilities

```typescript
import {
  createTransactionUtils,
  TransactionHelpers,
} from "./utils/transactions";
import { getBlockchainConfig } from "./sdk";

const txUtils = createTransactionUtils(getBlockchainConfig(config));

// Simple transfer
const transferTx = await txUtils.createTransferTransaction(
  fromPubkey,
  toPubkey,
  TransactionHelpers.solToLamports(1.5), // 1.5 SOL
  "Transfer memo",
);

// Send with retry logic
const result = await txUtils.sendAndConfirmTransaction(transferTx, [signer]);

// Transaction builder pattern
const tx = await txUtils
  .createTransactionBuilder(userPubkey)
  .addTransfer(recipientPubkey, TransactionHelpers.solToLamports(0.5))
  .addMemo("Payment for services")
  .build();
```

### WebSocket Connections

```typescript
import { createWebSocketManager } from "./sdk";

const wsManager = createWebSocketManager(config);

// Connect to live data
const connection = wsManager.getConnection("live-data");

connection.onMessage("update", (data) => {
  console.log("Live update:", data);
});

connection.connect();
```

## 🎨 Customizing the Wallet Component

```tsx
import { ReusableWalletConnect } from './components/ReusableWalletConnect';

// Full customization
<ReusableWalletConnect
  config={config}
  showBalance={true}
  showAirdrop={config.blockchain.network === 'devnet'}
  showExplorer={true}
  className="my-custom-wallet-styles"
  onWalletConnect={(pubkey) => {
    console.log('User connected:', pubkey);
    // Update your app state
  }}
  onBalanceUpdate={(balance) => {
    console.log('New balance:', balance);
    // Update UI or trigger actions
  }}
  onWalletDisconnect={() => {
    console.log('User disconnected');
    // Clean up state
  }}
/>

// Minimal setup
<ReusableWalletConnect config={blockchainConfig} />
```

## 🛠️ Development Tips

### 1. Environment Variables

```bash
# .env
REACT_APP_API_URL=https://your-api.com
REACT_APP_API_KEY=your-api-key
REACT_APP_NETWORK=devnet
REACT_APP_RPC_URL=https://api.devnet.solana.com
REACT_APP_PROGRAM_ID=your-program-id
```

### 2. Error Handling

```typescript
import {
  TransactionError,
  InsufficientFundsError,
  NetworkError,
} from "./utils/transactions";

try {
  const result = await vaultService.deposit(userPubkey, amount);
} catch (error) {
  if (error instanceof InsufficientFundsError) {
    alert("Not enough SOL for this transaction");
  } else if (error instanceof NetworkError) {
    alert("Network error - please check your connection");
  } else {
    console.error("Transaction failed:", error);
  }
}
```

### 3. Type Safety

```typescript
import type {
  AtomikConfig,
  BlockchainConfig,
  ApiConfig,
  TransactionResult,
} from "./sdk";

// All interfaces are exported for type safety
function handleConfig(config: AtomikConfig) {
  // TypeScript will enforce correct structure
}
```

### 4. Tree Shaking

Import only what you need:

```typescript
// Good - only imports what you use
import { createApiClient, createVaultService } from "./sdk";

// Less optimal - imports everything
import * as SDK from "./sdk";
```

## 🚀 Production Checklist

- [ ] Replace devnet with mainnet configuration
- [ ] Set proper API endpoints and keys
- [ ] Configure production RPC endpoints (Helius, QuickNode, etc.)
- [ ] Test all wallet connections (Phantom, Solflare, Ledger)
- [ ] Implement proper error boundaries
- [ ] Add transaction confirmation UI
- [ ] Set up monitoring and analytics
- [ ] Test with rate limiting scenarios
- [ ] Implement proper loading states
- [ ] Add transaction fee estimation

## 📖 Advanced Usage

### Custom Program Integration

```typescript
import {
  TransactionBuilder,
  createTransactionUtils,
} from "./utils/transactions";
import { createGenericConfig } from "./sdk";

// Configure for your program
const config = createGenericConfig({
  blockchain: {
    programId: "YOUR_CUSTOM_PROGRAM_ID",
    // ... other config
  },
});

// Use with your program instructions
const txUtils = createTransactionUtils(config.blockchain);
const builder = txUtils.createTransactionBuilder(userPubkey);

// Add your custom instruction
builder.addInstruction(yourCustomInstruction);
const tx = await builder.build();
```

### Multi-Program Support

```typescript
// Different configs for different programs
const gameConfig = createGenericConfig({
  blockchain: { programId: "GAME_PROGRAM_ID" /* ... */ },
});

const nftConfig = createGenericConfig({
  blockchain: { programId: "NFT_PROGRAM_ID" /* ... */ },
});

const gameSDK = createAtomikSDK(gameConfig);
const nftSDK = createAtomikSDK(nftConfig);
```

## 🤝 Contributing

This SDK is designed to be:

- **Modular**: Use individual services or the complete SDK
- **Type-Safe**: Full TypeScript support with proper interfaces
- **Extensible**: Easy to add new services and utilities
- **Framework-Agnostic**: Works with any React setup

Need help or want to contribute? The code is well-documented and follows clean architecture principles!
