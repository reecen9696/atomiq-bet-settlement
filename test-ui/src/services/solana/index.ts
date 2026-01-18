/**
 * Solana service main module
 * Re-exports all functionality from modular structure
 */

// Re-export types
export * from "./types";

// Re-export utilities
export {
  parseVaultAccount,
  parseCasinoAccount,
  parseAllowanceAccount,
  parseAllowanceNonceRegistryAccount,
  i64ToLeBytes,
  u64ToLeBytes,
  anchorDiscriminator,
  buildIxData,
  createUniqueMemoInstruction,
  sleep,
  isRateLimitError,
  withRateLimitRetry,
} from "./utils";

// Re-export PDA derivation
export { PDADerivation } from "./pda";

// For now, we keep the main SolanaService in the parent file
// and gradually migrate methods to the new structure
