# 🚀 Atomik SDK - Complete Implementation Guide

**Version**: 2.0  
**Date**: January 29, 2026  
**Status**: Production Ready

## 📖 Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [SDK Configuration](#sdk-configuration)
5. [Core Services](#core-services)
6. [Wallet Integration](#wallet-integration)
7. [Transaction Management](#transaction-management)
8. [Error Handling](#error-handling)
9. [Production Deployment](#production-deployment)
10. [Advanced Usage](#advanced-usage)
11. [Migration Guide](#migration-guide)
12. [Troubleshooting](#troubleshooting)

---

## 🎯 Overview

The Atomik SDK is a **production-ready, blockchain-agnostic React SDK** for building Solana applications. It provides:

- **Unified Configuration**: Works with any Solana program/API combination
- **Type Safety**: Full TypeScript support with comprehensive interfaces
- **Modular Design**: Use individual services or the complete SDK
- **Backward Compatibility**: Existing code continues to work unchanged
- **Production Features**: Error handling, retry logic, transaction confirmation
- **Real-time Support**: WebSocket connections with auto-reconnection

### ✅ **What's Included**

| Component             | Description                           | Use Case                   |
| --------------------- | ------------------------------------- | -------------------------- |
| **SDK Core**          | Configuration and service management  | All projects               |
| **API Client**        | REST client with retry logic          | Backend communication      |
| **Vault Service**     | Account management and PDA operations | Solana program interaction |
| **Allowance Service** | Delegation and authorization patterns | Multi-signature workflows  |
| **Betting Service**   | Gaming/transaction service example    | Casino/gaming applications |
| **WebSocket Manager** | Real-time connections                 | Live updates               |
| **Wallet Component**  | Complete wallet integration           | User authentication        |
| **Transaction Utils** | Helper functions and builders         | Transaction creation       |

---

## 🏗️ Architecture

### **Configuration System**

```
AtomikConfig (Generic)
├── blockchain: BlockchainConfig
│   ├── network: 'mainnet' | 'devnet' | 'testnet'
│   ├── programId: string
│   ├── rpcUrl: string
│   ├── commitment: Commitment
│   └── confirmTimeout: number
├── api: ApiConfig
│   ├── baseUrl: string
│   ├── apiKey?: string
│   ├── timeout: number
│   └── retryAttempts: number
└── websocket: WebSocketConfig
    ├── enabled: boolean
    ├── reconnectAttempts: number
    ├── reconnectDelay: number
    └── connectionTimeout: number
```

### **Service Architecture**

```
AtomikSDK
├── config: AtomikConfig
├── api: AtomikApiClient
├── vault: AtomikVaultService
├── allowance: AtomikAllowanceService
├── betting: AtomikBettingService (optional)
└── websocket: AtomikWebSocketManager
```

---

## 🚀 Quick Start

### **1. Installation**

```bash
# Install required dependencies
npm install @solana/web3.js @solana/wallet-adapter-base @solana/wallet-adapter-react @solana/wallet-adapter-react-ui @solana/wallet-adapter-wallets lucide-react

# Copy SDK files to your project
cp -r src/sdk/ your-project/src/
cp -r src/components/ReusableWalletConnect.tsx your-project/src/components/
cp -r src/utils/transactions.ts your-project/src/utils/
```

### **2. Basic Setup**

```tsx
// App.tsx
import { useMemo } from "react";
import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import { clusterApiUrl } from "@solana/web3.js";
import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";

// Import wallet adapter styles
import "@solana/wallet-adapter-react-ui/styles.css";

function App() {
  const network = WalletAdapterNetwork.Devnet;
  const endpoint = useMemo(() => clusterApiUrl(network), [network]);

  const wallets = useMemo(
    () => [new PhantomWalletAdapter(), new SolflareWalletAdapter()],
    [],
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <YourApp />
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
```

### **3. SDK Integration**

```tsx
// YourApp.tsx
import { createAtomikSDK } from "./sdk";
import { ReusableWalletConnect } from "./components/ReusableWalletConnect";

function YourApp() {
  // Configure SDK for your project
  const config = {
    blockchain: {
      network: "devnet" as const,
      programId: "YOUR_PROGRAM_ID",
      rpcUrl: "https://api.devnet.solana.com",
      commitment: "confirmed" as const,
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
  };

  const sdk = createAtomikSDK(config);

  return (
    <div className="app">
      <h1>My Solana App</h1>

      {/* Wallet connection component */}
      <ReusableWalletConnect
        config={config}
        showBalance={true}
        showAirdrop={true}
        onWalletConnect={(publicKey) => console.log("Connected:", publicKey)}
      />

      {/* Your app components */}
    </div>
  );
}
```

---

## ⚙️ SDK Configuration

### **Environment-Based Configuration**

```tsx
// .env
REACT_APP_NETWORK=devnet
REACT_APP_RPC_URL=https://api.devnet.solana.com
REACT_APP_PROGRAM_ID=YourProgramId
REACT_APP_API_URL=https://your-api.com
REACT_APP_API_KEY=your-api-key

// config.ts
import { createGenericConfig } from './sdk';

export const config = createGenericConfig({
  apiBaseUrl: process.env.REACT_APP_API_URL!,
  apiKey: process.env.REACT_APP_API_KEY,
  blockchainNetwork: process.env.REACT_APP_NETWORK!,
  blockchainRpcUrl: process.env.REACT_APP_RPC_URL!,
  programId: process.env.REACT_APP_PROGRAM_ID!,
});
```

### **Direct Configuration**

```tsx
import { AtomikConfig } from "./sdk";

const config: AtomikConfig = {
  blockchain: {
    network: "mainnet-beta",
    programId: "11111111111111111111111111111111",
    rpcUrl: "https://api.mainnet-beta.solana.com",
    commitment: "confirmed",
    confirmTimeout: 30000,
  },
  api: {
    baseUrl: "https://api.production.com",
    apiKey: "prod-api-key",
    timeout: 15000,
    retryAttempts: 5,
  },
  websocket: {
    enabled: true,
    reconnectAttempts: 10,
    reconnectDelay: 2000,
    connectionTimeout: 15000,
  },
};
```

### **Configuration Validation**

```tsx
import { AtomikConfig } from "./sdk";

function validateConfig(config: AtomikConfig): void {
  if (!config.blockchain.programId) {
    throw new Error("Program ID is required");
  }

  if (!config.api.baseUrl) {
    throw new Error("API base URL is required");
  }

  if (config.blockchain.network === "mainnet-beta" && !config.api.apiKey) {
    console.warn("API key recommended for mainnet");
  }
}
```

---

## 🔧 Core Services

### **1. API Client**

```tsx
import { createApiClient } from "./sdk";

const apiClient = createApiClient(config);

// GET request
const data = await apiClient.get<UserData>("/user/profile");

// POST request with data
const result = await apiClient.post<CreateResult>("/user/create", {
  name: "John Doe",
  email: "john@example.com",
});

// Error handling
try {
  const response = await apiClient.get("/protected-endpoint");
} catch (error) {
  if (error.message.includes("401")) {
    // Handle authentication error
  }
}
```

### **2. Vault Service**

```tsx
import { createVaultService } from "./sdk";
import { useWallet } from "@solana/wallet-adapter-react";

function VaultComponent() {
  const { publicKey } = useWallet();
  const vaultService = createVaultService(config);

  const initializeVault = async () => {
    if (!publicKey) return;

    try {
      // Derive vault PDA
      const vaultPda = await vaultService.deriveVaultPDA(publicKey.toBase58());

      // Check if vault exists
      const vaultInfo = await vaultService.getVaultInfo(publicKey.toBase58());

      if (!vaultInfo.accountExists) {
        // Initialize vault with 1 SOL
        const transaction = await vaultService.initializeVault(
          publicKey.toBase58(),
          1.0,
        );
        console.log("Vault initialized:", transaction);
      }
    } catch (error) {
      console.error("Vault operation failed:", error);
    }
  };

  return <button onClick={initializeVault}>Initialize Vault</button>;
}
```

### **3. Transaction Utilities**

```tsx
import {
  createTransactionUtils,
  TransactionHelpers,
} from "./utils/transactions";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";

function TransferComponent() {
  const { publicKey, signTransaction } = useWallet();
  const { connection } = useConnection();

  const txUtils = createTransactionUtils(config.blockchain);

  const sendTransfer = async () => {
    if (!publicKey || !signTransaction) return;

    try {
      // Create transfer transaction
      const transaction = await txUtils.createTransferTransaction(
        publicKey,
        new PublicKey("RECIPIENT_ADDRESS"),
        TransactionHelpers.solToLamports(0.1), // 0.1 SOL
        "Payment for services",
      );

      // Sign transaction
      const signed = await signTransaction(transaction);

      // Send with retry logic
      const result = await txUtils.sendAndConfirmTransaction(signed, []);

      if (result.success) {
        console.log("Transfer successful:", result.signature);
      } else {
        console.error("Transfer failed:", result.error);
      }
    } catch (error) {
      console.error("Transaction error:", error);
    }
  };

  return <button onClick={sendTransfer}>Send 0.1 SOL</button>;
}
```

### **4. WebSocket Integration**

```tsx
import { createWebSocketManager } from "./sdk";
import { useEffect, useState } from "react";

function LiveDataComponent() {
  const [data, setData] = useState(null);
  const wsManager = createWebSocketManager(config);

  useEffect(() => {
    const connection = wsManager.getConnection("live-updates");

    // Handle messages
    connection.onMessage("update", (newData) => {
      setData(newData);
    });

    // Handle connection events
    connection.onConnect(() => {
      console.log("WebSocket connected");
    });

    connection.onDisconnect(() => {
      console.log("WebSocket disconnected");
    });

    // Connect
    connection.connect();

    // Cleanup
    return () => {
      connection.disconnect();
    };
  }, []);

  return (
    <div>
      <h3>Live Data</h3>
      <pre>{JSON.stringify(data, null, 2)}</pre>
    </div>
  );
}
```

---

## 💳 Wallet Integration

### **Basic Wallet Component**

```tsx
import { ReusableWalletConnect } from "./components/ReusableWalletConnect";

function App() {
  return (
    <ReusableWalletConnect
      config={config}
      showBalance={true}
      showAirdrop={config.blockchain.network === "devnet"}
      showExplorer={true}
      onWalletConnect={(publicKey) => {
        console.log("User connected:", publicKey);
        // Update application state
      }}
      onBalanceUpdate={(balance) => {
        console.log("Balance updated:", balance);
      }}
      onWalletDisconnect={() => {
        console.log("User disconnected");
        // Clear application state
      }}
    />
  );
}
```

### **Custom Wallet Integration**

```tsx
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

function CustomWalletComponent() {
  const { publicKey, connected, connecting, disconnect } = useWallet();

  return (
    <div className="wallet-section">
      {!connected ? (
        <WalletMultiButton />
      ) : (
        <div className="wallet-info">
          <p>Connected: {publicKey?.toBase58().slice(0, 8)}...</p>
          <button onClick={disconnect}>Disconnect</button>
        </div>
      )}

      {connecting && <p>Connecting...</p>}
    </div>
  );
}
```

### **Wallet State Management**

```tsx
import { useWallet } from "@solana/wallet-adapter-react";
import { createContext, useContext, useEffect, useState } from "react";

interface WalletContextType {
  isConnected: boolean;
  publicKey: string | null;
  balance: number | null;
}

const WalletContext = createContext<WalletContextType>({
  isConnected: false,
  publicKey: null,
  balance: null,
});

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const { publicKey, connected } = useWallet();
  const [balance, setBalance] = useState<number | null>(null);

  useEffect(() => {
    if (connected && publicKey) {
      // Fetch balance
      const txUtils = createTransactionUtils(config.blockchain);
      txUtils.getBalance(publicKey).then(setBalance);
    } else {
      setBalance(null);
    }
  }, [connected, publicKey]);

  return (
    <WalletContext.Provider
      value={{
        isConnected: connected,
        publicKey: publicKey?.toBase58() || null,
        balance,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
}

export const useWalletState = () => useContext(WalletContext);
```

---

## 🔄 Transaction Management

### **Transaction Builder Pattern**

```tsx
import { createTransactionUtils } from "./utils/transactions";
import { PublicKey } from "@solana/web3.js";

async function complexTransaction() {
  const txUtils = createTransactionUtils(config.blockchain);

  // Build transaction with multiple operations
  const transaction = await txUtils
    .createTransactionBuilder(userPublicKey)
    .addTransfer(
      new PublicKey("RECIPIENT_1"),
      TransactionHelpers.solToLamports(0.1),
    )
    .addTransfer(
      new PublicKey("RECIPIENT_2"),
      TransactionHelpers.solToLamports(0.2),
    )
    .addMemo("Batch payment")
    .build();

  return transaction;
}
```

### **Transaction Confirmation**

```tsx
import { TransactionConfirmationComponent } from "./components/TransactionConfirmation";

function TransactionFlow() {
  const [txSignature, setTxSignature] = useState<string | null>(null);
  const [isConfirming, setIsConfirming] = useState(false);

  const handleTransaction = async (transaction: Transaction) => {
    setIsConfirming(true);

    try {
      const result = await txUtils.sendAndConfirmTransaction(
        transaction,
        signers,
      );

      if (result.success) {
        setTxSignature(result.signature!);
        await txUtils.waitForConfirmation(result.signature!);
      }
    } catch (error) {
      console.error("Transaction failed:", error);
    } finally {
      setIsConfirming(false);
    }
  };

  return (
    <div>
      {isConfirming && <div>Confirming transaction...</div>}
      {txSignature && (
        <div>
          Transaction confirmed:
          <a
            href={TransactionHelpers.getExplorerUrl(
              txSignature,
              config.blockchain.network,
              "tx",
            )}
          >
            View on Explorer
          </a>
        </div>
      )}
    </div>
  );
}
```

### **Batch Transactions**

```tsx
async function batchTransactions() {
  const txUtils = createTransactionUtils(config.blockchain);
  const transactions: Transaction[] = [];

  // Create multiple transactions
  for (const recipient of recipients) {
    const tx = await txUtils.createTransferTransaction(
      userPublicKey,
      new PublicKey(recipient.address),
      TransactionHelpers.solToLamports(recipient.amount),
    );
    transactions.push(tx);
  }

  // Send transactions with delay
  const results = [];
  for (const tx of transactions) {
    const result = await txUtils.sendAndConfirmTransaction(tx, [signer]);
    results.push(result);

    // Add delay between transactions to avoid rate limiting
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  return results;
}
```

---

## 🚨 Error Handling

### **Typed Error Classes**

```tsx
import {
  TransactionError,
  InsufficientFundsError,
  NetworkError,
} from "./utils/transactions";

async function handleTransactionWithErrors() {
  try {
    const result = await vaultService.deposit(userPublicKey, amount);
    return result;
  } catch (error) {
    if (error instanceof InsufficientFundsError) {
      // Handle insufficient funds
      showError("Not enough SOL for this transaction");
      return null;
    }

    if (error instanceof NetworkError) {
      // Handle network issues
      showError("Network error - please check your connection");
      return null;
    }

    if (error instanceof TransactionError) {
      // Handle transaction-specific errors
      showError(`Transaction failed: ${error.message}`);
      if (error.signature) {
        console.log("Failed transaction signature:", error.signature);
      }
      return null;
    }

    // Handle unknown errors
    console.error("Unexpected error:", error);
    showError("An unexpected error occurred");
    return null;
  }
}
```

### **Global Error Handler**

```tsx
import { createContext, useContext, useState } from "react";

interface ErrorContextType {
  error: string | null;
  showError: (message: string) => void;
  clearError: () => void;
}

const ErrorContext = createContext<ErrorContextType>({
  error: null,
  showError: () => {},
  clearError: () => {},
});

export function ErrorProvider({ children }: { children: React.ReactNode }) {
  const [error, setError] = useState<string | null>(null);

  const showError = (message: string) => {
    setError(message);
    // Auto-clear after 5 seconds
    setTimeout(() => setError(null), 5000);
  };

  const clearError = () => setError(null);

  return (
    <ErrorContext.Provider value={{ error, showError, clearError }}>
      {children}
      {error && (
        <div className="error-toast">
          <p>{error}</p>
          <button onClick={clearError}>✕</button>
        </div>
      )}
    </ErrorContext.Provider>
  );
}

export const useError = () => useContext(ErrorContext);
```

### **Retry Logic**

```tsx
async function withRetry<T>(
  operation: () => Promise<T>,
  maxAttempts: number = 3,
  delay: number = 1000,
): Promise<T> {
  let lastError: Error;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;

      if (attempt === maxAttempts) {
        throw lastError;
      }

      // Exponential backoff
      await new Promise((resolve) =>
        setTimeout(resolve, delay * Math.pow(2, attempt - 1)),
      );
    }
  }

  throw lastError!;
}

// Usage
const result = await withRetry(
  () => apiClient.get("/unstable-endpoint"),
  3, // 3 attempts
  1000, // 1 second base delay
);
```

---

## 🚀 Production Deployment

### **Environment Configuration**

```bash
# Production .env
REACT_APP_NETWORK=mainnet-beta
REACT_APP_RPC_URL=https://your-production-rpc.com
REACT_APP_PROGRAM_ID=YourProgramId
REACT_APP_API_URL=https://api.production.com
REACT_APP_API_KEY=production-api-key

# Development .env
REACT_APP_NETWORK=devnet
REACT_APP_RPC_URL=https://api.devnet.solana.com
REACT_APP_PROGRAM_ID=DevProgramId
REACT_APP_API_URL=http://localhost:8080
REACT_APP_API_KEY=dev-api-key
```

### **Production Configuration**

```tsx
// config/production.ts
import { AtomikConfig } from "../sdk";

export const productionConfig: AtomikConfig = {
  blockchain: {
    network: "mainnet-beta",
    programId: process.env.REACT_APP_PROGRAM_ID!,
    rpcUrl: process.env.REACT_APP_RPC_URL!,
    commitment: "confirmed",
    confirmTimeout: 60000, // 60 seconds for mainnet
  },
  api: {
    baseUrl: process.env.REACT_APP_API_URL!,
    apiKey: process.env.REACT_APP_API_KEY!,
    timeout: 30000, // 30 seconds for production
    retryAttempts: 5, // More retries for production
  },
  websocket: {
    enabled: true,
    reconnectAttempts: 10,
    reconnectDelay: 2000,
    connectionTimeout: 15000,
  },
};
```

### **Performance Monitoring**

```tsx
import { useEffect } from "react";

// Transaction performance monitoring
function TransactionMonitor() {
  useEffect(() => {
    const originalSendTransaction = txUtils.sendAndConfirmTransaction;

    txUtils.sendAndConfirmTransaction = async (
      transaction,
      signers,
      options,
    ) => {
      const startTime = Date.now();

      try {
        const result = await originalSendTransaction(
          transaction,
          signers,
          options,
        );
        const duration = Date.now() - startTime;

        // Log successful transaction
        console.log(`Transaction completed in ${duration}ms`, {
          signature: result.signature,
          duration,
          success: result.success,
        });

        // Send to analytics
        if (window.gtag) {
          window.gtag("event", "transaction_success", {
            duration,
            network: config.blockchain.network,
          });
        }

        return result;
      } catch (error) {
        const duration = Date.now() - startTime;

        console.error(`Transaction failed after ${duration}ms`, error);

        if (window.gtag) {
          window.gtag("event", "transaction_error", {
            duration,
            error: error.message,
            network: config.blockchain.network,
          });
        }

        throw error;
      }
    };
  }, []);

  return null;
}
```

### **Health Checks**

```tsx
import { useEffect, useState } from "react";

function HealthCheck() {
  const [status, setStatus] = useState({
    rpc: "unknown",
    api: "unknown",
    websocket: "unknown",
  });

  useEffect(() => {
    const checkHealth = async () => {
      // Check RPC
      try {
        await txUtils.getConnection().getSlot();
        setStatus((prev) => ({ ...prev, rpc: "healthy" }));
      } catch {
        setStatus((prev) => ({ ...prev, rpc: "unhealthy" }));
      }

      // Check API
      try {
        await apiClient.get("/health");
        setStatus((prev) => ({ ...prev, api: "healthy" }));
      } catch {
        setStatus((prev) => ({ ...prev, api: "unhealthy" }));
      }

      // Check WebSocket
      const wsConnection = wsManager.getConnection("health-check");
      wsConnection.onConnect(() => {
        setStatus((prev) => ({ ...prev, websocket: "healthy" }));
        wsConnection.disconnect();
      });
      wsConnection.connect();
    };

    checkHealth();
    const interval = setInterval(checkHealth, 30000); // Check every 30 seconds

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="health-status">
      <div>RPC: {status.rpc}</div>
      <div>API: {status.api}</div>
      <div>WebSocket: {status.websocket}</div>
    </div>
  );
}
```

---

## 🔬 Advanced Usage

### **Custom Program Integration**

```tsx
// Define your program instructions
import {
  TransactionInstruction,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";

interface CustomProgramService {
  createCustomInstruction(
    userPublicKey: PublicKey,
    amount: number,
  ): TransactionInstruction;
}

class MyCustomProgramService implements CustomProgramService {
  private programId: PublicKey;

  constructor(programId: string) {
    this.programId = new PublicKey(programId);
  }

  createCustomInstruction(
    userPublicKey: PublicKey,
    amount: number,
  ): TransactionInstruction {
    // Create your program-specific instruction
    return new TransactionInstruction({
      keys: [
        { pubkey: userPublicKey, isSigner: true, isWritable: true },
        // Add other required accounts
      ],
      programId: this.programId,
      data: Buffer.from([
        /* your instruction data */
      ]),
    });
  }
}

// Usage with SDK
const customService = new MyCustomProgramService(config.blockchain.programId);

async function executeCustomTransaction() {
  const instruction = customService.createCustomInstruction(
    userPublicKey,
    1000,
  );

  const transaction = await txUtils
    .createTransactionBuilder(userPublicKey)
    .addInstruction(instruction)
    .addMemo("Custom program interaction")
    .build();

  return await txUtils.sendAndConfirmTransaction(transaction, [signer]);
}
```

### **Multi-Wallet Support**

```tsx
import { useWallet } from "@solana/wallet-adapter-react";
import { useState, useEffect } from "react";

function MultiWalletManager() {
  const { wallets, wallet, select } = useWallet();
  const [availableWallets, setAvailableWallets] = useState<string[]>([]);

  useEffect(() => {
    // Filter available wallets
    const installed = wallets
      .filter((w) => w.readyState === "Installed")
      .map((w) => w.adapter.name);

    setAvailableWallets(installed);
  }, [wallets]);

  return (
    <div className="multi-wallet-selector">
      <h3>Select Wallet</h3>
      {availableWallets.map((walletName) => (
        <button
          key={walletName}
          onClick={() => {
            const selectedWallet = wallets.find(
              (w) => w.adapter.name === walletName,
            );
            if (selectedWallet) select(selectedWallet.adapter.name);
          }}
          className={wallet?.adapter.name === walletName ? "active" : ""}
        >
          {walletName}
        </button>
      ))}
    </div>
  );
}
```

### **Cross-Program Transactions**

```tsx
async function crossProgramTransaction() {
  const vaultService = createVaultService(config);
  const customService = new MyCustomProgramService(config.blockchain.programId);

  // Build transaction with multiple program interactions
  const transaction = await txUtils
    .createTransactionBuilder(userPublicKey)
    .addInstruction(
      await vaultService.createDepositInstruction(userPublicKey, 1.0),
    )
    .addInstruction(customService.createCustomInstruction(userPublicKey, 1000))
    .addMemo("Cross-program transaction")
    .build();

  return await txUtils.sendAndConfirmTransaction(transaction, [signer]);
}
```

---

## 📋 Migration Guide

### **From Legacy Casino Code**

```tsx
// OLD (Legacy Casino Code)
import { createAtomikConfig, createAtomikSDK } from "./sdk";

const config = createAtomikConfig(); // Environment-based
const sdk = createAtomikSDK(config);

// NEW (Continues to work unchanged)
import { createAtomikConfig, createAtomikSDK } from "./sdk";

const config = createAtomikConfig(); // Still works exactly the same
const sdk = createAtomikSDK(config);
```

### **To Modern Configuration**

```tsx
// NEW (Recommended for new projects)
import { createAtomikSDK, AtomikConfig } from "./sdk";

const config: AtomikConfig = {
  blockchain: {
    network: "mainnet-beta",
    programId: "YOUR_PROGRAM_ID",
    rpcUrl: "https://api.mainnet-beta.solana.com",
    commitment: "confirmed",
    confirmTimeout: 30000,
  },
  api: {
    baseUrl: "https://your-api.com",
    apiKey: "your-key",
    timeout: 10000,
    retryAttempts: 3,
  },
  websocket: {
    enabled: true,
    reconnectAttempts: 5,
    reconnectDelay: 1000,
    connectionTimeout: 10000,
  },
};

const sdk = createAtomikSDK(config);
```

### **Component Migration**

```tsx
// OLD
import { WalletConnect } from "./components/WalletConnect";

function App() {
  return <WalletConnect />; // Still works
}

// NEW (More features)
import { ReusableWalletConnect } from "./components/ReusableWalletConnect";

function App() {
  return (
    <ReusableWalletConnect
      config={config}
      onWalletConnect={(publicKey) => {
        // Handle connection
      }}
    />
  );
}
```

---

## 🔧 Troubleshooting

### **Common Issues**

#### **1. Transaction Failures**

```tsx
// Problem: Transactions failing silently
// Solution: Check transaction status and implement proper error handling

const result = await txUtils.sendAndConfirmTransaction(transaction, signers);

if (!result.success) {
  console.error("Transaction failed:", result.error);

  // Common failure reasons:
  if (result.error?.includes("insufficient funds")) {
    // User doesn't have enough SOL
  } else if (result.error?.includes("blockhash")) {
    // Blockhash expired, retry transaction
  } else if (result.error?.includes("simulation failed")) {
    // Transaction would fail, check program logic
  }
}
```

#### **2. RPC Rate Limiting**

```tsx
// Problem: Getting 429 errors from RPC
// Solution: Implement proper retry logic with backoff

const txUtilsWithRetry = createTransactionUtils({
  ...config.blockchain,
  // Use custom RPC with higher rate limits
  rpcUrl: "https://your-premium-rpc.com",
});

// Also implement exponential backoff
async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
): Promise<T> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      if (i === maxRetries - 1) throw error;

      const delay = Math.pow(2, i) * 1000; // Exponential backoff
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }
  throw new Error("Max retries exceeded");
}
```

#### **3. WebSocket Connection Issues**

```tsx
// Problem: WebSocket disconnecting frequently
// Solution: Check network stability and implement proper reconnection

const wsManager = createWebSocketManager({
  ...config,
  websocket: {
    ...config.websocket,
    reconnectAttempts: 10, // Increase retry attempts
    reconnectDelay: 5000, // Increase delay between retries
  },
});

// Monitor connection status
const connection = wsManager.getConnection("main");

connection.onDisconnect(() => {
  console.log("WebSocket disconnected, checking network...");

  // Check if it's a network issue
  fetch("/api/health")
    .then(() => console.log("Network is fine, WebSocket issue"))
    .catch(() => console.log("Network issue detected"));
});
```

#### **4. Wallet Connection Issues**

```tsx
// Problem: Wallet not connecting or connecting to wrong network
// Solution: Verify wallet adapter setup and network configuration

import { useWallet, useConnection } from "@solana/wallet-adapter-react";

function WalletDebugger() {
  const { wallet, connected, connecting, connect } = useWallet();
  const { connection } = useConnection();

  useEffect(() => {
    console.log("Wallet status:", {
      wallet: wallet?.adapter.name,
      connected,
      connecting,
      endpoint: connection.rpcEndpoint,
    });
  }, [wallet, connected, connecting, connection]);

  const checkNetwork = async () => {
    try {
      const version = await connection.getVersion();
      console.log("RPC Version:", version);

      const slot = await connection.getSlot();
      console.log("Current slot:", slot);
    } catch (error) {
      console.error("RPC check failed:", error);
    }
  };

  return (
    <div>
      <button onClick={checkNetwork}>Check Network</button>
      <pre>{JSON.stringify({ connected, connecting }, null, 2)}</pre>
    </div>
  );
}
```

### **Performance Optimization**

```tsx
// 1. Lazy load heavy components
const VaultManager = lazy(() => import("./components/VaultManager"));

// 2. Memoize expensive calculations
const memoizedConfig = useMemo(() => createAtomikSDK(config), [config]);

// 3. Debounce user inputs
const debouncedAmount = useDebounce(amount, 500);

// 4. Use React.memo for components that don't change often
const WalletDisplay = React.memo(({ publicKey }: { publicKey: string }) => {
  return <div>Wallet: {publicKey}</div>;
});

// 5. Optimize transaction building
const transactionCache = new Map();

function getCachedTransaction(key: string) {
  if (transactionCache.has(key)) {
    return transactionCache.get(key);
  }

  // Build transaction and cache it
  const transaction = buildTransaction();
  transactionCache.set(key, transaction);

  return transaction;
}
```

### **Debugging Tools**

```tsx
// Enable debug mode in development
if (process.env.NODE_ENV === "development") {
  // Log all SDK operations
  window.atomikDebug = {
    logApiCalls: true,
    logTransactions: true,
    logWebSocketEvents: true,
  };

  // Add global error handler
  window.addEventListener("unhandledrejection", (event) => {
    console.error("Unhandled promise rejection:", event.reason);
  });
}

// Transaction debugger
function TransactionDebugger() {
  const [logs, setLogs] = useState<string[]>([]);

  useEffect(() => {
    const originalLog = console.log;
    console.log = (...args) => {
      originalLog(...args);
      setLogs((prev) => [...prev, args.join(" ")]);
    };

    return () => {
      console.log = originalLog;
    };
  }, []);

  return (
    <div className="debug-panel">
      <h3>Debug Logs</h3>
      <pre>{logs.slice(-20).join("\n")}</pre>
    </div>
  );
}
```

---

## 📚 API Reference

### **Core Types**

```tsx
// Configuration interfaces
interface AtomikConfig {
  blockchain: BlockchainConfig;
  api: ApiConfig;
  websocket: WebSocketConfig;
}

interface BlockchainConfig {
  network: "mainnet-beta" | "devnet" | "testnet";
  programId: string;
  rpcUrl: string;
  commitment: "processed" | "confirmed" | "finalized";
  confirmTimeout: number;
}

interface ApiConfig {
  baseUrl: string;
  apiKey?: string;
  timeout: number;
  retryAttempts: number;
}

interface WebSocketConfig {
  enabled: boolean;
  reconnectAttempts: number;
  reconnectDelay: number;
  connectionTimeout: number;
}

// Transaction result types
interface TransactionResult {
  success: boolean;
  signature?: string;
  error?: string;
  blockhash?: string;
}

// API response types
interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}
```

### **Factory Functions**

```tsx
// SDK creation
function createAtomikSDK(config: AtomikConfig | AtomikSolanaConfig): AtomikSDK;

// Individual services
function createApiClient(
  config: AtomikConfig | AtomikSolanaConfig,
): AtomikApiClient;
function createVaultService(
  config: AtomikConfig | AtomikSolanaConfig,
): AtomikVaultService;
function createAllowanceService(
  config: AtomikConfig | AtomikSolanaConfig,
): AtomikAllowanceService;
function createBettingService(
  config: AtomikConfig | AtomikSolanaConfig,
  apiClient: AtomikApiClient,
): AtomikBettingService;
function createWebSocketManager(
  config: AtomikConfig | AtomikSolanaConfig,
): AtomikWebSocketManager;

// Transaction utilities
function createTransactionUtils(config: BlockchainConfig): TransactionUtils;

// Configuration helpers
function createGenericConfig(
  env: GenericEnv,
  overrides?: Partial<AtomikConfig>,
): AtomikConfig;
function createConfigFromParams(params: ConfigParams): AtomikConfig;
function getBlockchainConfig(
  config: AtomikConfig | AtomikSolanaConfig,
): BlockchainConfig;
function getApiConfig(config: AtomikConfig | AtomikSolanaConfig): ApiConfig;
```

---

## 🎯 Best Practices

### **1. Configuration Management**

- Use environment variables for sensitive data
- Validate configuration on startup
- Use different configs for different environments
- Cache configuration objects to avoid recreation

### **2. Error Handling**

- Use typed error classes for better debugging
- Implement global error boundaries
- Log errors with context information
- Provide user-friendly error messages

### **3. Performance**

- Lazy load heavy components
- Memoize expensive calculations
- Use transaction caching where appropriate
- Implement proper loading states

### **4. Security**

- Never expose private keys in client code
- Use environment variables for API keys
- Validate user inputs before transactions
- Implement proper CSRF protection

### **5. Testing**

- Mock external dependencies (RPC, APIs)
- Test error conditions thoroughly
- Use integration tests for critical flows
- Test wallet connection/disconnection scenarios

### **6. Monitoring**

- Log transaction performance metrics
- Monitor RPC endpoint health
- Track error rates and types
- Set up alerts for critical failures

---

## 🔗 Resources

### **Documentation**

- [Solana Web3.js Documentation](https://solana-labs.github.io/solana-web3.js/)
- [Wallet Adapter Documentation](https://github.com/solana-labs/wallet-adapter)
- [Solana Developer Resources](https://docs.solana.com/)

### **Tools**

- [Solana Explorer](https://explorer.solana.com/)
- [Solana Beach](https://solanabeach.io/)
- [Phantom Wallet](https://phantom.app/)
- [Solflare Wallet](https://solflare.com/)

### **RPC Providers**

- [QuickNode](https://quicknode.com/)
- [Alchemy](https://www.alchemy.com/)
- [Helius](https://helius.xyz/)
- [GenesysGo](https://genesysgo.com/)

---

**📞 Support**

For questions or issues, please refer to the troubleshooting section above or check the project's README.md for additional support information.

---

_Last updated: January 29, 2026_
