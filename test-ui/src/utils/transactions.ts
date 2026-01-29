import {
  Connection,
  Transaction,
  PublicKey,
  SystemProgram,
  Keypair,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import type { BlockchainConfig } from "../sdk/env";

export interface TransactionOptions {
  skipPreflight?: boolean;
  preflightCommitment?: "processed" | "confirmed" | "finalized";
  maxRetries?: number;
  timeout?: number;
}

export interface TransactionResult {
  success: boolean;
  signature?: string;
  error?: string;
  blockhash?: string;
}

/**
 * Reusable transaction building and execution utilities
 * Works with any Solana project, not tied to specific program IDs
 */
export class TransactionUtils {
  private connection: Connection;
  private config: BlockchainConfig;

  constructor(config: BlockchainConfig) {
    this.config = config;
    this.connection = new Connection(config.rpcUrl, {
      commitment: config.commitment,
    });
  }

  /**
   * Get the connection instance
   */
  getConnection(): Connection {
    return this.connection;
  }

  /**
   * Create a basic transfer transaction
   */
  async createTransferTransaction(
    fromPubkey: PublicKey,
    toPubkey: PublicKey,
    lamports: number,
    memo?: string,
  ): Promise<Transaction> {
    const transaction = new Transaction();

    // Add transfer instruction
    transaction.add(
      SystemProgram.transfer({
        fromPubkey,
        toPubkey,
        lamports,
      }),
    );

    // Add memo if provided
    if (memo) {
      const { createMemoInstruction } = await import("../sdk/utils/memo");
      transaction.add(createMemoInstruction(memo));
    }

    // Set recent blockhash
    const { blockhash } = await this.connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = fromPubkey;

    return transaction;
  }

  /**
   * Send and confirm a transaction with retry logic
   */
  async sendAndConfirmTransaction(
    transaction: Transaction,
    signers: Keypair[],
    options: TransactionOptions = {},
  ): Promise<TransactionResult> {
    const {
      skipPreflight = false,
      preflightCommitment = this.config.commitment,
      maxRetries = 3,
    } = options;

    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        // Get fresh blockhash for retry attempts
        if (attempt > 1) {
          const { blockhash } = await this.connection.getLatestBlockhash();
          transaction.recentBlockhash = blockhash;
        }

        const signature = await sendAndConfirmTransaction(
          this.connection,
          transaction,
          signers,
          {
            skipPreflight,
            preflightCommitment,
          },
        );

        return {
          success: true,
          signature,
          blockhash: transaction.recentBlockhash,
        };
      } catch (error) {
        lastError = error as Error;
        console.warn(
          `Transaction attempt ${attempt} failed:`,
          lastError.message,
        );

        if (attempt === maxRetries) {
          break;
        }

        // Wait before retry (exponential backoff)
        await new Promise((resolve) => setTimeout(resolve, 1000 * attempt));
      }
    }

    return {
      success: false,
      error: lastError?.message || "Transaction failed after retries",
    };
  }

  /**
   * Estimate transaction fee
   */
  async estimateTransactionFee(transaction: Transaction): Promise<number> {
    try {
      const feeCalculator = await this.connection.getFeeForMessage(
        transaction.compileMessage(),
        this.config.commitment,
      );
      return feeCalculator.value || 5000; // Fallback to 5000 lamports
    } catch (error) {
      console.warn("Fee estimation failed, using default:", error);
      return 5000; // Default fee
    }
  }

  /**
   * Wait for transaction confirmation with timeout
   */
  async waitForConfirmation(
    signature: string,
    commitment: "processed" | "confirmed" | "finalized" = "confirmed",
    timeout: number = this.config.confirmTimeout || 30000,
  ): Promise<boolean> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeout) {
      try {
        const status = await this.connection.getSignatureStatus(signature);

        if (
          status.value?.confirmationStatus === commitment ||
          status.value?.confirmationStatus === "finalized"
        ) {
          return true;
        }

        if (status.value?.err) {
          throw new Error(
            `Transaction failed: ${JSON.stringify(status.value.err)}`,
          );
        }
      } catch (error) {
        console.warn("Error checking transaction status:", error);
      }

      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    return false;
  }

  /**
   * Get account balance in SOL
   */
  async getBalance(publicKey: PublicKey | string): Promise<number> {
    const key =
      typeof publicKey === "string" ? new PublicKey(publicKey) : publicKey;
    const lamports = await this.connection.getBalance(key);
    return lamports / 1_000_000_000; // Convert to SOL
  }

  /**
   * Request airdrop (devnet only)
   */
  async requestAirdrop(
    publicKey: PublicKey | string,
    amountSol: number = 1,
  ): Promise<string> {
    if (this.config.network !== "devnet") {
      throw new Error("Airdrop only available on devnet");
    }

    const key =
      typeof publicKey === "string" ? new PublicKey(publicKey) : publicKey;
    const lamports = amountSol * 1_000_000_000;

    const signature = await this.connection.requestAirdrop(key, lamports);
    await this.waitForConfirmation(signature);
    return signature;
  }

  /**
   * Create a transaction builder for chaining operations
   */
  createTransactionBuilder(feePayer: PublicKey): TransactionBuilder {
    return new TransactionBuilder(this.connection, feePayer);
  }
}

/**
 * Transaction builder for chaining multiple instructions
 */
export class TransactionBuilder {
  private connection: Connection;
  private transaction: Transaction;
  private feePayer: PublicKey;

  constructor(connection: Connection, feePayer: PublicKey) {
    this.connection = connection;
    this.feePayer = feePayer;
    this.transaction = new Transaction();
  }

  /**
   * Add an instruction to the transaction
   */
  addInstruction(instruction: TransactionInstruction): this {
    this.transaction.add(instruction);
    return this;
  }

  /**
   * Add multiple instructions
   */
  addInstructions(instructions: TransactionInstruction[]): this {
    this.transaction.add(...instructions);
    return this;
  }

  /**
   * Add a memo to the transaction
   */
  async addMemo(memo: string): Promise<this> {
    const { createMemoInstruction } = await import("../sdk/utils/memo");
    this.transaction.add(createMemoInstruction(memo));
    return this;
  }

  /**
   * Add a transfer instruction
   */
  addTransfer(toPubkey: PublicKey, lamports: number): this {
    this.transaction.add(
      SystemProgram.transfer({
        fromPubkey: this.feePayer,
        toPubkey,
        lamports,
      }),
    );
    return this;
  }

  /**
   * Build the final transaction
   */
  async build(): Promise<Transaction> {
    const { blockhash } = await this.connection.getLatestBlockhash();
    this.transaction.recentBlockhash = blockhash;
    this.transaction.feePayer = this.feePayer;
    return this.transaction;
  }
}

/**
 * Factory function to create TransactionUtils from different config types
 */
export function createTransactionUtils(
  config: BlockchainConfig,
): TransactionUtils {
  return new TransactionUtils(config);
}

/**
 * Error types for better error handling
 */
export class TransactionError extends Error {
  constructor(
    message: string,
    public signature?: string,
    public code?: string,
  ) {
    super(message);
    this.name = "TransactionError";
  }
}

export class InsufficientFundsError extends TransactionError {
  constructor(required: number, available: number) {
    super(
      `Insufficient funds: required ${required} SOL, available ${available} SOL`,
    );
    this.name = "InsufficientFundsError";
  }
}

export class NetworkError extends TransactionError {
  constructor(
    message: string,
    public networkUrl?: string,
  ) {
    super(message);
    this.name = "NetworkError";
  }
}

/**
 * Utility functions for common operations
 */
export const TransactionHelpers = {
  /**
   * Convert lamports to SOL
   */
  lamportsToSol: (lamports: number): number => lamports / 1_000_000_000,

  /**
   * Convert SOL to lamports
   */
  solToLamports: (sol: number): number => Math.floor(sol * 1_000_000_000),

  /**
   * Format SOL amount for display
   */
  formatSol: (sol: number, decimals: number = 4): string =>
    sol.toFixed(decimals),

  /**
   * Validate Solana public key
   */
  isValidPublicKey: (key: string): boolean => {
    try {
      new PublicKey(key);
      return true;
    } catch {
      return false;
    }
  },

  /**
   * Get explorer URL for transaction or address
   */
  getExplorerUrl: (
    item: string,
    network: "mainnet" | "devnet" | "testnet" = "mainnet",
    type: "tx" | "address" = "address",
  ): string => {
    const base = "https://explorer.solana.com";
    const cluster = network === "mainnet" ? "" : `?cluster=${network}`;
    return `${base}/${type}/${item}${cluster}`;
  },
};
