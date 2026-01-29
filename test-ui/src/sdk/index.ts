// Core configuration
export { 
  createAtomikConfig, 
  getAtomikUiEnv,
  type AtomikConfig,
  type AtomikUiEnv 
} from './env';

// Memo utilities for transaction descriptions
export {
  createMemoInstruction,
  MemoMessages,
  MEMO_PROGRAM_ID,
  truncateMemo
} from './utils/memo';

// API client
export {
  AtomikApiClient,
  createApiClient,
  type ApiResponse,
  type CoinflipRequest,
  type CoinflipResult,
  type Settlement,
  type RecentGame,
  type PaginatedGames
} from './api/client';

// WebSocket management
export {
  AtomikWebSocketManager,
  WebSocketConnection,
  createWebSocketManager,
  type WebSocketMessage,
  type CasinoStatsMessage,
  type RecentWinMessage,
  type BlockUpdateMessage,
  type AtomikWebSocketMessage
} from './websocket/manager';

// Solana services
export {
  AtomikVaultService,
  createVaultService,
  type VaultOperations
} from './solana/vault';

export {
  AtomikAllowanceService,
  createAllowanceService,
  type AllowanceOperations
} from './solana/allowance';

export {
  AtomikBettingService,
  createBettingService,
  type BettingOperations
} from './solana/betting';

/**
 * Main SDK factory that creates all services with shared configuration
 */
import type { AtomikConfig } from './env';
import type { AtomikApiClient } from './api/client';
import type { AtomikVaultService } from './solana/vault';
import type { AtomikAllowanceService } from './solana/allowance';
import type { AtomikBettingService } from './solana/betting';
import type { AtomikWebSocketManager } from './websocket/manager';

export interface AtomikSDK {
  config: AtomikConfig;
  api: AtomikApiClient;
  vault: AtomikVaultService;
  allowance: AtomikAllowanceService;
  betting: AtomikBettingService;
  websocket: AtomikWebSocketManager;
}

/**
 * Create a complete SDK instance with all services
 */
import { createAtomikConfig } from './env';
import { createApiClient } from './api/client';
import { createVaultService } from './solana/vault';
import { createAllowanceService } from './solana/allowance';
import { createBettingService } from './solana/betting';
import { createWebSocketManager } from './websocket/manager';

export function createAtomikSDK(configOverrides: Partial<AtomikConfig> = {}): AtomikSDK {
  const config = createAtomikConfig(configOverrides);
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
 */
export const AtomikSDKFactory = {
  /**
   * Create just the API client
   */
  createApiOnly: (configOverrides: Partial<AtomikConfig> = {}) => {
    const config = createAtomikConfig(configOverrides);
    return createApiClient(config);
  },

  /**
   * Create vault operations only
   */
  createVaultOnly: (configOverrides: Partial<AtomikConfig> = {}) => {
    const config = createAtomikConfig(configOverrides);
    return createVaultService(config);
  },

  /**
   * Create betting service only
   */
  createBettingOnly: (configOverrides: Partial<AtomikConfig> = {}) => {
    const config = createAtomikConfig(configOverrides);
    const api = createApiClient(config);
    return createBettingService(config, api);
  },

  /**
   * Create WebSocket manager only
   */
  createWebSocketOnly: (configOverrides: Partial<AtomikConfig> = {}) => {
    const config = createAtomikConfig(configOverrides);
    return createWebSocketManager(config);
  },
};