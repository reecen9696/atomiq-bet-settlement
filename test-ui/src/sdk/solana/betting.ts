import { Connection, PublicKey } from "@solana/web3.js";
import type {
  AtomikConfig,
  AtomikSolanaConfig,
  BlockchainConfig,
} from "../env";
import { getBlockchainConfig } from "../env";
import { PDADerivation } from "../../services/solana/pda";
import { parseCasinoAccount } from "../../services/solana/utils";
import type { CasinoAccountState } from "../../services/solana/types";
import type { AtomikApiClient } from "../api/client";

export interface BettingOperations {
  // Casino management
  deriveCasinoPDA(): Promise<string>;
  deriveCasinoVaultPDA(): Promise<string>;

  getCasinoInfo(connection?: Connection): Promise<{
    casinoPda: string;
    accountExists: boolean;
    casinoData: CasinoAccountState | null;
  }>;

  // Betting operations
  placeCoinflipBet(params: {
    userPublicKey: string;
    choice: "heads" | "tails";
    amount: number;
    vaultPda?: string;
    allowancePda?: string;
  }): Promise<{
    gameId: string;
    outcome: "heads" | "tails";
    won: boolean;
    amount: number;
  }>;

  // Game result checking
  getGameResult(gameId: string): Promise<{
    gameId: string;
    outcome: "heads" | "tails";
    won: boolean;
    amount: number;
    timestamp: string;
  } | null>;
}

/**
 * Service for casino betting operations
 * Handles coinflip games and result checking
 */
export class AtomikBettingService implements BettingOperations {
  private connection: Connection;
  private blockchainConfig: BlockchainConfig;
  private pda: PDADerivation;
  private apiClient: AtomikApiClient;
  private vaultProgramId: PublicKey;

  constructor(
    config: AtomikConfig | AtomikSolanaConfig,
    apiClient: AtomikApiClient,
  ) {
    this.blockchainConfig = getBlockchainConfig(config);
    this.connection = new Connection(this.blockchainConfig.rpcUrl, {
      commitment: this.blockchainConfig.commitment,
    });
    this.apiClient = apiClient;
    this.vaultProgramId = new PublicKey(this.blockchainConfig.programId);
    this.pda = new PDADerivation(this.vaultProgramId);
  }

  /**
   * Derive casino PDA
   */
  async deriveCasinoPDA(): Promise<string> {
    return this.pda.deriveCasinoPDA().toBase58();
  }

  /**
   * Derive casino vault PDA
   */
  async deriveCasinoVaultPDA(): Promise<string> {
    return this.pda.deriveCasinoVaultPDA().toBase58();
  }

  /**
   * Get casino account information
   */
  async getCasinoInfo(connection?: Connection) {
    const conn = connection || this.connection;
    const casinoPda = await this.deriveCasinoPDA();

    try {
      const accountInfo = await conn.getAccountInfo(new PublicKey(casinoPda));

      if (!accountInfo) {
        return {
          casinoPda,
          accountExists: false,
          casinoData: null,
        };
      }

      const casinoData = parseCasinoAccount(accountInfo.data);

      return {
        casinoPda,
        accountExists: true,
        casinoData,
      };
    } catch (error) {
      return {
        casinoPda,
        accountExists: false,
        casinoData: null,
        error: (error as Error).message,
      };
    }
  }

  /**
   * Place a coinflip bet
   */
  async placeCoinflipBet(params: {
    userPublicKey: string;
    choice: "heads" | "tails";
    amount: number;
    vaultPda?: string;
    allowancePda?: string;
  }) {
    const { userPublicKey, choice, amount, vaultPda, allowancePda } = params;

    // Derive vault PDA if not provided
    const userVaultPda =
      vaultPda ||
      (await this.pda.deriveVaultPDA(new PublicKey(userPublicKey))).toBase58();

    // Make API call to place bet
    const response = await this.apiClient.playCoinflip({
      choice,
      amount,
      userPubkey: userPublicKey,
      vaultPda: userVaultPda,
      allowancePda,
    });

    if (!response.success || !response.data) {
      throw new Error(response.error || "Failed to place coinflip bet");
    }

    return response.data;
  }

  /**
   * Get game result by ID
   */
  async getGameResult(gameId: string) {
    const response = await this.apiClient.getGameResult(gameId);

    if (!response.success) {
      if (response.error?.includes("not found")) {
        return null;
      }
      throw new Error(response.error || "Failed to get game result");
    }

    return response.data || null;
  }

  /**
   * Get recent games for a user
   */
  async getRecentGames(cursor?: string) {
    const response = await this.apiClient.getRecentGames(cursor);

    if (!response.success || !response.data) {
      throw new Error(response.error || "Failed to get recent games");
    }

    return response.data;
  }

  /**
   * Get pending settlements for a user
   */
  async getPendingSettlements(userPublicKey: string) {
    const response = await this.apiClient.getPendingSettlements(userPublicKey);

    if (!response.success || !response.data) {
      throw new Error(response.error || "Failed to get pending settlements");
    }

    return response.data;
  }

  /**
   * Get detailed settlement information
   */
  async getSettlement(settlementId: string) {
    const response = await this.apiClient.getSettlement(settlementId);

    if (!response.success) {
      if (response.error?.includes("not found")) {
        return null;
      }
      throw new Error(response.error || "Failed to get settlement");
    }

    return response.data || null;
  }

  /**
   * Wait for game settlement with polling
   */
  async waitForGameSettlement(
    gameId: string,
    timeoutMs: number = 30000,
  ): Promise<{
    gameId: string;
    outcome: "heads" | "tails";
    won: boolean;
    amount: number;
    timestamp: string;
  }> {
    const startTime = Date.now();
    const pollInterval = 2000; // 2 seconds

    while (Date.now() - startTime < timeoutMs) {
      try {
        const result = await this.getGameResult(gameId);
        if (result) {
          return result;
        }
      } catch (error) {
        console.warn("Error polling for game result:", error);
      }

      // Wait before next poll
      await new Promise((resolve) => setTimeout(resolve, pollInterval));
    }

    throw new Error(`Game settlement timeout after ${timeoutMs}ms`);
  }
}

/**
 * Factory function to create a betting service
 * Supports both new generic config and legacy Solana config
 */
export function createBettingService(
  config: AtomikConfig | AtomikSolanaConfig,
  apiClient: AtomikApiClient,
): AtomikBettingService {
  return new AtomikBettingService(config, apiClient);
}
