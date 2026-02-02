// Core configuration - both generic and legacy
export {
  createAtomikConfig,
  createGenericConfig,
  createConfigFromParams,
  getAtomikUiEnv,
  getBlockchainConfig,
  getApiConfig,
  type AtomikConfig,
  type AtomikSolanaConfig, // Legacy support
  type BlockchainConfig,
  type ApiConfig,
  type WebSocketConfig,
  type AtomikUiEnv,
} from "./env";

// Memo utilities for transaction descriptions
export {
  createMemoInstruction,
  MemoMessages,
  MEMO_PROGRAM_ID,
  truncateMemo,
} from "./utils/memo";

// API client
export {
  AtomikApiClient,
  createApiClient,
  type ApiResponse,
  type CoinflipRequest,
  type CoinflipResult,
  type Settlement,
  type RecentGame,
  type PaginatedGames,
} from "./api/client";

// WebSocket management
export {
  AtomikWebSocketManager,
  WebSocketConnection,
  createWebSocketManager,
  type WebSocketMessage,
  type CasinoStatsMessage,
  type RecentWinMessage,
  type BlockUpdateMessage,
  type AtomikWebSocketMessage,
} from "./websocket/manager";

// Solana services
export {
  AtomikVaultService,
  createVaultService,
  type VaultOperations,
} from "./solana/vault";

export {
  AtomikAllowanceService,
  createAllowanceService,
  type AllowanceOperations,
} from "./solana/allowance";

export {
  AtomikBettingService,
  createBettingService,
  type BettingOperations,
} from "./solana/betting";

/**
 * Main SDK factory that creates all services with shared configuration
 * Supports both generic AtomikConfig and legacy AtomikSolanaConfig
 */
import type { AtomikConfig, AtomikSolanaConfig } from "./env";
import type { AtomikApiClient } from "./api/client";
import type { AtomikVaultService } from "./solana/vault";
import type { AtomikAllowanceService } from "./solana/allowance";
import type { AtomikBettingService } from "./solana/betting";
import type { AtomikWebSocketManager } from "./websocket/manager";

export interface AtomikSDK {
  config: AtomikConfig | AtomikSolanaConfig;
  api: AtomikApiClient;
  vault: AtomikVaultService;
  allowance: AtomikAllowanceService;
  betting: AtomikBettingService;
  websocket: AtomikWebSocketManager;
}

/**
 * Create a complete SDK instance with all services
 * Supports both new AtomikConfig and legacy configuration patterns
 */
import { createAtomikConfig } from "./env";
import { createApiClient } from "./api/client";
import { createVaultService } from "./solana/vault";
import { createAllowanceService } from "./solana/allowance";
import { createBettingService } from "./solana/betting";
import { createWebSocketManager } from "./websocket/manager";

export function createAtomikSDK(
  configOrOverrides?: AtomikConfig | AtomikSolanaConfig | Partial<AtomikConfig>,
): AtomikSDK {
  // If config looks like a complete config object, use it directly
  const config =
    configOrOverrides &&
    ("api" in configOrOverrides || "apiBaseUrl" in configOrOverrides)
      ? (configOrOverrides as AtomikConfig | AtomikSolanaConfig)
      : createAtomikConfig((configOrOverrides as Partial<AtomikConfig>) || {});

  const api = createApiClient(config);
  const vault = createVaultService(config);
  const allowance = createAllowanceService(config);
  const betting = createBettingService(config, api);
  const websocket = createWebSocketManager(config);

  return {
    config,
    api,
    vault,
    allowance,
    betting,
    websocket,
  };
}

/**
 * Convenience functions for creating individual services
 * All support both AtomikConfig and legacy AtomikSolanaConfig
 */
export const AtomikSDKFactory = {
  /**
   * Create just the API client
   */
  createApiOnly: (
    configOrOverrides?:
      | AtomikConfig
      | AtomikSolanaConfig
      | Partial<AtomikConfig>,
  ) => {
    const config =
      configOrOverrides &&
      ("api" in configOrOverrides || "apiBaseUrl" in configOrOverrides)
        ? (configOrOverrides as AtomikConfig | AtomikSolanaConfig)
        : createAtomikConfig(
            (configOrOverrides as Partial<AtomikConfig>) || {},
          );
    return createApiClient(config);
  },

  /**
   * Create vault operations only
   */
  createVaultOnly: (
    configOrOverrides?:
      | AtomikConfig
      | AtomikSolanaConfig
      | Partial<AtomikConfig>,
  ) => {
    const config =
      configOrOverrides &&
      ("api" in configOrOverrides || "apiBaseUrl" in configOrOverrides)
        ? (configOrOverrides as AtomikConfig | AtomikSolanaConfig)
        : createAtomikConfig(
            (configOrOverrides as Partial<AtomikConfig>) || {},
          );
    return createVaultService(config);
  },

  /**
   * Create betting service only
   */
  createBettingOnly: (
    configOrOverrides?:
      | AtomikConfig
      | AtomikSolanaConfig
      | Partial<AtomikConfig>,
  ) => {
    const config =
      configOrOverrides &&
      ("api" in configOrOverrides || "apiBaseUrl" in configOrOverrides)
        ? (configOrOverrides as AtomikConfig | AtomikSolanaConfig)
        : createAtomikConfig(
            (configOrOverrides as Partial<AtomikConfig>) || {},
          );
    const api = createApiClient(config);
    return createBettingService(config, api);
  },

  /**
   * Create WebSocket manager only
   */
  createWebSocketOnly: (
    configOrOverrides?:
      | AtomikConfig
      | AtomikSolanaConfig
      | Partial<AtomikConfig>,
  ) => {
    const config =
      configOrOverrides &&
      ("api" in configOrOverrides || "apiBaseUrl" in configOrOverrides)
        ? (configOrOverrides as AtomikConfig | AtomikSolanaConfig)
        : createAtomikConfig(
            (configOrOverrides as Partial<AtomikConfig>) || {},
          );
    return createWebSocketManager(config);
  },
};
