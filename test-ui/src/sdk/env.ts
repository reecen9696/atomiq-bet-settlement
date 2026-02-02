type RequiredString = string;

/**
 * Generic blockchain configuration interface
 * Can be used with any blockchain, not just Solana
 */
export interface BlockchainConfig {
  rpcUrl: string;
  network: string;
  programId: string;
  commitment: "confirmed" | "finalized" | "processed";
  confirmTimeout: number;
}

/**
 * Generic API configuration interface
 */
export interface ApiConfig {
  baseUrl: string;
  apiKey?: string;
  timeout: number;
  retryAttempts: number;
}

/**
 * Generic WebSocket configuration interface
 */
export interface WebSocketConfig {
  enabled: boolean;
  reconnectAttempts: number;
  reconnectDelay: number;
  connectionTimeout: number;
}

/**
 * Main SDK configuration interface
 * Generic enough to work with different blockchains and APIs
 */
export interface AtomikConfig {
  api: ApiConfig;
  blockchain: BlockchainConfig;
  websocket: WebSocketConfig;
}

/**
 * Legacy Solana-specific config for backward compatibility
 */
export interface AtomikSolanaConfig {
  api: {
    baseUrl: string;
    settlementApiKey?: string;
    timeout: number;
    retryAttempts: number;
  };
  solana: {
    rpcUrl: string;
    network: string;
    programId: string;
    commitment: "confirmed" | "finalized" | "processed";
    confirmTimeout: number;
  };
  websocket: {
    enabled: boolean;
    reconnectAttempts: number;
    reconnectDelay: number;
    connectionTimeout: number;
  };
}

/**
 * Generic environment interface that works with any bundler
 * Not tied to Vite-specific import.meta.env
 */
export type GenericEnv = {
  apiBaseUrl: RequiredString;
  apiKey?: string;
  blockchainRpcUrl: RequiredString;
  blockchainNetwork: RequiredString;
  programId: RequiredString;
};

/**
 * Legacy Vite-specific environment (for backward compatibility)
 */
export type AtomikUiEnv = {
  apiBaseUrl: RequiredString;
  settlementApiKey?: string;
  solanaRpcUrl: RequiredString;
  solanaNetwork: RequiredString;
  vaultProgramId: RequiredString;
};

function required(name: string, value: unknown): string {
  if (typeof value === "string" && value.trim().length > 0) return value;
  throw new Error(`Missing required env var: ${name}`);
}

/**
 * Generic configuration factory that works with any environment
 * Not tied to Vite or any specific bundler
 */
export function createGenericConfig(
  env: GenericEnv,
  overrides: Partial<AtomikConfig> = {},
): AtomikConfig {
  const defaultConfig: AtomikConfig = {
    api: {
      baseUrl: env.apiBaseUrl,
      apiKey: env.apiKey,
      timeout: 30000, // 30 seconds
      retryAttempts: 3,
    },
    blockchain: {
      rpcUrl: env.blockchainRpcUrl,
      network: env.blockchainNetwork,
      programId: env.programId,
      commitment: "confirmed",
      confirmTimeout: 60000, // 60 seconds
    },
    websocket: {
      enabled: true,
      reconnectAttempts: 10,
      reconnectDelay: 1000, // Start with 1 second
      connectionTimeout: 10000, // 10 seconds
    },
  };

  return mergeConfig(defaultConfig, overrides);
}

/**
 * Factory for creating configuration from manual parameters
 * Useful for testing or when environment variables aren't available
 */
export function createConfigFromParams(params: {
  apiBaseUrl: string;
  blockchainRpcUrl: string;
  blockchainNetwork?: string;
  programId: string;
  apiKey?: string;
  overrides?: Partial<AtomikConfig>;
}): AtomikConfig {
  const env: GenericEnv = {
    apiBaseUrl: params.apiBaseUrl,
    blockchainRpcUrl: params.blockchainRpcUrl,
    blockchainNetwork: params.blockchainNetwork || "devnet",
    programId: params.programId,
    apiKey: params.apiKey,
  };

  return createGenericConfig(env, params.overrides);
}

/**
 * Reads the Vite env used by this test UI and creates a complete configuration.
 *
 * Keep this in one place so copy/pasting the Solana + API logic into another
 * app is straightforward.
 */
export function getAtomikUiEnv(): AtomikUiEnv {
  const env = import.meta.env;

  return {
    apiBaseUrl: required(
      "VITE_API_BASE_URL",
      env.VITE_API_BASE_URL || "http://localhost:8080",
    ),
    settlementApiKey:
      typeof env.VITE_SETTLEMENT_API_KEY === "string"
        ? env.VITE_SETTLEMENT_API_KEY
        : undefined,
    solanaRpcUrl: required(
      "VITE_SOLANA_RPC_URL",
      env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com",
    ),
    solanaNetwork:
      (typeof env.VITE_SOLANA_NETWORK === "string" &&
        env.VITE_SOLANA_NETWORK) ||
      "devnet",
    vaultProgramId: required(
      "VITE_VAULT_PROGRAM_ID",
      env.VITE_VAULT_PROGRAM_ID,
    ),
  };
}

/**
 * Creates a complete Atomik configuration from environment variables.
 * This is the main configuration factory for the SDK.
 *
 * @deprecated Use createGenericConfig for better reusability across projects
 */
export function createAtomikConfig(
  overrides: Partial<AtomikSolanaConfig> = {},
): AtomikSolanaConfig {
  const env = getAtomikUiEnv();

  const defaultConfig: AtomikSolanaConfig = {
    api: {
      baseUrl: env.apiBaseUrl,
      settlementApiKey: env.settlementApiKey,
      timeout: 30000, // 30 seconds
      retryAttempts: 3,
    },
    solana: {
      rpcUrl: env.solanaRpcUrl,
      network: env.solanaNetwork,
      programId: env.vaultProgramId,
      commitment: "confirmed",
      confirmTimeout: 60000, // 60 seconds
    },
    websocket: {
      enabled: true,
      reconnectAttempts: 10,
      reconnectDelay: 1000, // Start with 1 second
      connectionTimeout: 10000, // 10 seconds
    },
  };

  return mergeSolanaConfig(defaultConfig, overrides);
}

function mergeConfig(
  base: AtomikConfig,
  overrides: Partial<AtomikConfig>,
): AtomikConfig {
  return {
    api: { ...base.api, ...overrides.api },
    blockchain: { ...base.blockchain, ...overrides.blockchain },
    websocket: { ...base.websocket, ...overrides.websocket },
  };
}

function mergeSolanaConfig(
  base: AtomikSolanaConfig,
  overrides: Partial<AtomikSolanaConfig>,
): AtomikSolanaConfig {
  return {
    api: { ...base.api, ...overrides.api },
    solana: { ...base.solana, ...overrides.solana },
    websocket: { ...base.websocket, ...overrides.websocket },
  };
}

/**
 * Configuration adapter functions for backward compatibility
 */

/**
 * Extract blockchain configuration from either AtomikConfig or AtomikSolanaConfig
 */
export function getBlockchainConfig(
  config: AtomikConfig | AtomikSolanaConfig,
): BlockchainConfig {
  if ("blockchain" in config) {
    return config.blockchain;
  }
  // Legacy AtomikSolanaConfig
  return {
    network: config.solana.network,
    programId: config.solana.programId,
    rpcUrl: config.solana.rpcUrl,
    commitment: config.solana.commitment,
    confirmTimeout: 30000, // Default timeout for legacy configs
  };
}

/**
 * Extract API configuration from either AtomikConfig or AtomikSolanaConfig
 */
export function getApiConfig(
  config: AtomikConfig | AtomikSolanaConfig,
): ApiConfig {
  if ("api" in config && "baseUrl" in config.api) {
    return config.api;
  }
  // Legacy AtomikSolanaConfig with mixed api structure
  const legacyConfig = config as AtomikSolanaConfig;
  return {
    baseUrl:
      legacyConfig.api?.baseUrl ||
      (legacyConfig as any).apiBaseUrl ||
      "http://localhost:8080",
    apiKey:
      legacyConfig.api?.settlementApiKey ||
      (legacyConfig as any).settlementApiKey,
    timeout: legacyConfig.api?.timeout || 30000,
    retryAttempts: legacyConfig.api?.retryAttempts || 3,
  };
}
