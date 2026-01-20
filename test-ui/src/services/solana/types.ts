/**
 * Shared TypeScript types for Solana vault program accounts
 */

export type VaultAccountState = {
  owner: string;
  casino: string;
  bump: number;
  solBalanceLamports: bigint;
  createdAt: bigint;
  lastActivity: bigint;
};

export type CasinoAccountState = {
  authority: string;
  processor: string;
  treasury: string;
  bump: number;
  vaultAuthorityBump: number;
  paused: boolean;
  totalBets: bigint;
  totalVolumeLamports: bigint;
  createdAt: bigint;
};

export type AllowanceAccountState = {
  user: string;
  casino: string;
  tokenMint: string;
  amountLamports: bigint;
  spentLamports: bigint;
  expiresAt: bigint;
  createdAt: bigint;
  nonce: bigint;
  revoked: boolean;
  bump: number;
  lastSpentAt: bigint;
  spendCount: number;
  remainingLamports: bigint;
};

export type AllowanceNonceRegistryState = {
  user: string;
  casino: string;
  nextNonce: bigint;
  bump: number;
};
