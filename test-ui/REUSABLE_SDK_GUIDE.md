# Reusable Atomik SDK Guide

## Overview

The Atomik SDK has been refactored to be completely reusable across different blockchain projects while maintaining 100% backward compatibility with the existing casino UI.

## Key Improvements

### ✅ Configuration Abstraction

- **Generic Configuration**: New `AtomikConfig` interface supports any blockchain/API setup
- **Backward Compatible**: Legacy `AtomikSolanaConfig` still works seamlessly
- **Flexible Configuration**: Supports environment variables, direct params, or config objects

### ✅ Service Architecture

- **Modular Services**: Each service (vault, allowance, betting, API, WebSocket) can be used independently
- **Config Adapters**: Automatic translation between old and new config formats
- **Factory Functions**: Easy service creation with `createXService()` functions

### ✅ API Client Abstraction

- **Generic REST Client**: Works with any REST API, not just Atomik's
- **Configurable Endpoints**: Easy to adapt for different backend APIs
- **Error Handling**: Built-in retry logic and error management

### ✅ WebSocket Manager

- **Generic WebSocket Wrapper**: Automatic reconnection, message routing
- **Configurable URLs**: Adapts to any WebSocket backend
- **Type-Safe Messages**: Strong typing for WebSocket message handling

## Usage Examples

### For New Projects (Generic Configuration)

```typescript
import { createAtomikSDK, createGenericConfig } from "./sdk";

// Option 1: Quick setup with environment detection
const sdk = createAtomikSDK();

// Option 2: Custom configuration for your project
const config = createGenericConfig({
  blockchain: {
    network: "devnet",
    programId: "YOUR_PROGRAM_ID",
    rpcUrl: "https://api.devnet.solana.com",
  },
  api: {
    baseUrl: "https://your-api.com",
    apiKey: "your-api-key",
    timeout: 10000,
    retryAttempts: 3,
  },
  websocket: {
    enabled: true,
    reconnectInterval: 5000,
    maxReconnectAttempts: 5,
  },
});

const sdk = createAtomikSDK(config);

// Use individual services
const apiClient = sdk.api;
const vaultService = sdk.vault;
const bettingService = sdk.betting;
```

### For Legacy Projects (Existing Casino UI)

```typescript
import { createAtomikSDK, createAtomikConfig } from "./sdk";

// Existing code continues to work unchanged
const sdk = createAtomikSDK({
  apiBaseUrl: "http://localhost:8080",
  settlementApiKey: "your-key",
});
```

### Individual Service Usage

```typescript
import {
  createApiClient,
  createVaultService,
  createGenericConfig,
} from "./sdk";

// Create just what you need
const config = createGenericConfig({
  api: { baseUrl: "https://api.example.com", apiKey: "key" },
  blockchain: {
    network: "mainnet",
    rpcUrl: "https://api.mainnet-beta.solana.com",
  },
});

const apiClient = createApiClient(config);
const vaultService = createVaultService(config);
```

## Migration Guide

### For Existing Casino Code

**No changes required!** All existing code continues to work:

```typescript
// This still works exactly the same
const config = createAtomikConfig();
const sdk = createAtomikSDK(config);
```

### For New Projects

Use the new generic configuration system:

```typescript
import { createGenericConfig, createAtomikSDK } from "./sdk";

const config = createGenericConfig({
  blockchain: {
    network: "your-network",
    programId: "your-program-id",
    rpcUrl: "your-rpc-url",
  },
  api: {
    baseUrl: "your-api-base-url",
    apiKey: "your-api-key",
  },
});

const sdk = createAtomikSDK(config);
```

## Exported Types & Functions

### Configuration

- `AtomikConfig` - New generic configuration interface
- `AtomikSolanaConfig` - Legacy configuration (backward compatibility)
- `BlockchainConfig` - Blockchain-specific settings
- `ApiConfig` - REST API settings
- `WebSocketConfig` - WebSocket connection settings

### Factory Functions

- `createAtomikSDK()` - Complete SDK with all services
- `createGenericConfig()` - New generic configuration builder
- `createConfigFromParams()` - Direct parameter configuration
- `createAtomikConfig()` - Legacy configuration builder (backward compatible)

### Individual Services

- `createApiClient()` - REST API client
- `createVaultService()` - Blockchain vault operations
- `createAllowanceService()` - Allowance management
- `createBettingService()` - Betting/gaming operations
- `createWebSocketManager()` - WebSocket connections

### Utilities

- `AtomikSDKFactory` - Convenience methods for creating single services
- `MemoMessages` - Transaction memo utilities
- Various TypeScript types for API responses, WebSocket messages, etc.

## Benefits

1. **🔄 Reusable Across Projects**: Use the same SDK for any Solana-based project
2. **⚡ Zero Breaking Changes**: Existing casino code works without modifications
3. **🧩 Modular**: Use individual services or the complete SDK
4. **📝 Type Safe**: Full TypeScript support with proper type inference
5. **🛠️ Configurable**: Easy to adapt for different APIs, networks, and requirements
6. **🔌 Extensible**: Clean architecture makes it easy to add new features

## Next Steps

1. **Extract Wallet Component**: Make WalletConnect.tsx blockchain-agnostic
2. **Abstract Transaction Patterns**: Extract memo instruction utilities
3. **Create Project Template**: Scaffold new projects with this SDK
4. **Documentation**: Add comprehensive API documentation
