type RequiredString = string;

export interface AtomikConfig {
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
  };
}

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
 */
export function createAtomikConfig(
  overrides: Partial<AtomikConfig> = {},
): AtomikConfig {
  const env = getAtomikUiEnv();

  const defaultConfig: AtomikConfig = {
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
    },
  };

  return mergeConfig(defaultConfig, overrides);
}

function mergeConfig(
  base: AtomikConfig,
  overrides: Partial<AtomikConfig>,
): AtomikConfig {
  return {
    api: { ...base.api, ...overrides.api },
    solana: { ...base.solana, ...overrides.solana },
    websocket: { ...base.websocket, ...overrides.websocket },
  };
}
