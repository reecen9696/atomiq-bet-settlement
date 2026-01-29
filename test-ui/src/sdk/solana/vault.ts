import {
  Connection,
  PublicKey,
} from "@solana/web3.js";
import type { AtomikConfig } from "../env";
import { PDADerivation } from "../../services/solana/pda";
import { parseVaultAccount } from "../../services/solana/utils";
import type { VaultAccountState } from "../../services/solana/types";
import { MemoMessages } from "../utils/memo";

export interface VaultOperations {
  // PDA derivation
  deriveVaultPDA(userPublicKey: string): Promise<string>;
  deriveVaultAuthorityPDA(): Promise<string>;

  // Account operations
  getVaultInfo(params: {
    userPublicKey: string;
    connection?: Connection;
  }): Promise<{
    vaultPda: string;
    accountExists: boolean;
    vaultData: VaultAccountState | null;
  }>;

  // Vault transactions
  initializeUserVault(params: {
    userPublicKey: string;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string>;

  depositSol(params: {
    userPublicKey: string;
    amount: number;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string>;

  withdrawSol(params: {
    userPublicKey: string;
    amount: number;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string>;

  // Balance checking
  getBalance(publicKey: string, connection?: Connection): Promise<number>;
}

/**
 * Service for managing user vault operations on Solana
 * Handles vault initialization, deposits, and withdrawals
 */
export class AtomikVaultService implements VaultOperations {
  private connection: Connection;
  private config: AtomikConfig;
  private pda: PDADerivation;
  private vaultProgramId: PublicKey;

  constructor(config: AtomikConfig) {
    this.config = config;
    this.connection = new Connection(config.solana.rpcUrl, {
      commitment: config.solana.commitment,
    });
    this.vaultProgramId = new PublicKey(config.solana.programId);
    this.pda = new PDADerivation(this.vaultProgramId);
  }

  /**
   * Derive vault PDA for a user
   */
  async deriveVaultPDA(userPublicKey: string): Promise<string> {
    return this.pda.deriveVaultPDA(new PublicKey(userPublicKey)).toBase58();
  }

  /**
   * Derive vault authority PDA
   */
  async deriveVaultAuthorityPDA(): Promise<string> {
    return this.pda.deriveVaultAuthorityPDA().toBase58();
  }

  /**
   * Get vault information for a user
   */
  async getVaultInfo(params: {
    userPublicKey: string;
    connection?: Connection;
  }) {
    const { userPublicKey, connection = this.connection } = params;
    const vaultPda = await this.deriveVaultPDA(userPublicKey);

    try {
      const accountInfo = await connection.getAccountInfo(
        new PublicKey(vaultPda),
      );

      if (!accountInfo) {
        return {
          vaultPda,
          accountExists: false,
          vaultData: null,
        };
      }

      const vaultData = parseVaultAccount(accountInfo.data);

      return {
        vaultPda,
        accountExists: true,
        vaultData,
      };
    } catch (error) {
      return {
        vaultPda,
        accountExists: false,
        vaultData: null,
        error: (error as Error).message,
      };
    }
  }

  /**
   * Initialize a user vault
   */
  async initializeUserVault(_params: {
    userPublicKey: string;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string> {
    // Create memo instruction first (so it appears prominently in wallet)
    // const _memoInstruction = createMemoInstruction(MemoMessages.initializeVault());
    
    // This would build the full transaction with:
    // 1. Memo instruction (first)
    // 2. Initialize vault instruction
    // For now, returning a placeholder that would be implemented with the full logic
    throw new Error("Not implemented - would use full SolanaService logic with memo: '" + MemoMessages.initializeVault() + "'");
  }

  /**
   * Deposit SOL into user vault
   */
  async depositSol(params: {
    userPublicKey: string;
    amount: number;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string> {
    const { amount } = params;
    
    // Create memo instruction first (so it appears prominently in wallet)
    // const _memoInstruction = createMemoInstruction(MemoMessages.depositSol(amount));
    
    // This would build the full transaction with:
    // 1. Memo instruction (first) 
    // 2. Deposit instruction
    throw new Error("Not implemented - would use full SolanaService logic with memo: '" + MemoMessages.depositSol(amount) + "'");
  }

  /**
   * Withdraw SOL from user vault
   */
  async withdrawSol(params: {
    userPublicKey: string;
    amount: number;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string> {
    const { amount } = params;
    
    // Create memo instruction first (so it appears prominently in wallet)
    // const _memoInstruction = createMemoInstruction(MemoMessages.withdrawSol(amount));
    
    // This would build the full transaction with:
    // 1. Memo instruction (first)
    // 2. Withdraw instruction
    throw new Error("Not implemented - would use full SolanaService logic with memo: '" + MemoMessages.withdrawSol(amount) + "'");
  }

  /**
   * Get SOL balance for a public key
   */
  async getBalance(
    publicKey: string,
    connection?: Connection,
  ): Promise<number> {
    const conn = connection || this.connection;
    const balance = await conn.getBalance(new PublicKey(publicKey));
    return balance / 1e9; // Convert lamports to SOL
  }

  /**
   * Request airdrop for devnet testing
   */
  async requestAirdrop(publicKey: string, amount: number = 1): Promise<string> {
    if (this.config.solana.network !== "devnet") {
      throw new Error("Airdrop only available on devnet");
    }

    const signature = await this.connection.requestAirdrop(
      new PublicKey(publicKey),
      amount * 1e9, // Convert SOL to lamports
    );

    return signature;
  }
}

/**
 * Factory function to create a vault service
 */
export function createVaultService(config: AtomikConfig): AtomikVaultService {
  return new AtomikVaultService(config);
}
