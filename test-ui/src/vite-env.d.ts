/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string;
  readonly VITE_SETTLEMENT_API_KEY?: string;
  readonly VITE_SOLANA_RPC_URL: string;
  readonly VITE_VAULT_PROGRAM_ID: string;
  readonly VITE_PRIVY_APP_ID: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
