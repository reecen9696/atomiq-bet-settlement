import { Connection, PublicKey } from "@solana/web3.js";
import type { AtomikConfig } from "../env";
import { PDADerivation } from "../../services/solana/pda";
import {
  parseAllowanceAccount,
  parseAllowanceNonceRegistryAccount,
} from "../../services/solana/utils";
import type { AllowanceAccountState } from "../../services/solana/types";

export interface AllowanceOperations {
  // PDA derivation
  deriveAllowancePDA(params: {
    userPublicKey: string;
    spender: string;
    nonce: number;
  }): Promise<string>;

  deriveAllowanceNonceRegistryPDA(params: {
    userPublicKey: string;
    spender: string;
  }): Promise<string>;

  // Nonce management
  getNextAllowanceNonce(params: {
    userPublicKey: string;
    spender: string;
    connection?: Connection;
  }): Promise<number>;

  // Account operations
  getAllowanceInfo(
    allowancePda: string,
    connection?: Connection,
  ): Promise<{
    accountExists: boolean;
    allowanceData: AllowanceAccountState | null;
  }>;

  // Allowance transactions
  approveAllowance(params: {
    userPublicKey: string;
    spender: string;
    amount: number;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<{
    signature: string;
    allowancePda: string;
  }>;

  revokeAllowance(params: {
    userPublicKey: string;
    allowancePda: string;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string>;
}

/**
 * Service for managing SOL allowances on Solana
 * Handles allowance approval and revocation with nonce-based deterministic PDAs
 */
export class AtomikAllowanceService implements AllowanceOperations {
  private connection: Connection;
  private pda: PDADerivation;
  private vaultProgramId: PublicKey;

  constructor(config: AtomikConfig) {
    this.connection = new Connection(config.solana.rpcUrl, {
      commitment: config.solana.commitment,
    });
    this.vaultProgramId = new PublicKey(config.solana.programId);
    this.pda = new PDADerivation(this.vaultProgramId);
  }

  /**
   * Derive allowance PDA with nonce
   */
  async deriveAllowancePDA(params: {
    userPublicKey: string;
    spender: string;
    nonce: number;
  }): Promise<string> {
    const { userPublicKey, nonce } = params;
    const userPubkey = new PublicKey(userPublicKey);
    const casinoPda = this.pda.deriveCasinoPDA();

    return this.pda
      .deriveAllowancePDA(userPubkey, BigInt(nonce), casinoPda)
      .toBase58();
  }

  /**
   * Derive allowance nonce registry PDA
   */
  async deriveAllowanceNonceRegistryPDA(params: {
    userPublicKey: string;
    spender: string;
  }): Promise<string> {
    const { userPublicKey } = params;
    const userPubkey = new PublicKey(userPublicKey);
    const casinoPda = this.pda.deriveCasinoPDA();

    return this.pda
      .deriveAllowanceNonceRegistryPDA(userPubkey, casinoPda)
      .toBase58();
  }

  /**
   * Get the next available nonce for allowances
   */
  async getNextAllowanceNonce(params: {
    userPublicKey: string;
    spender: string;
    connection?: Connection;
  }): Promise<number> {
    const { userPublicKey, spender, connection = this.connection } = params;

    try {
      const registryPda = await this.deriveAllowanceNonceRegistryPDA({
        userPublicKey,
        spender,
      });

      const accountInfo = await connection.getAccountInfo(
        new PublicKey(registryPda),
      );

      if (!accountInfo) {
        // No registry exists yet, start with nonce 0
        return 0;
      }

      const registryData = parseAllowanceNonceRegistryAccount(accountInfo.data);
      return Number(registryData.nextNonce);
    } catch (error) {
      console.warn("Error fetching allowance nonce:", error);
      return 0;
    }
  }

  /**
   * Get allowance account information
   */
  async getAllowanceInfo(allowancePda: string, connection?: Connection) {
    const conn = connection || this.connection;

    try {
      const accountInfo = await conn.getAccountInfo(
        new PublicKey(allowancePda),
      );

      if (!accountInfo) {
        return {
          accountExists: false,
          allowanceData: null,
        };
      }

      const allowanceData = parseAllowanceAccount(accountInfo.data);

      return {
        accountExists: true,
        allowanceData,
      };
    } catch (error) {
      return {
        accountExists: false,
        allowanceData: null,
        error: (error as Error).message,
      };
    }
  }

  /**
   * Approve an allowance for a spender
   */
  async approveAllowance(_params: {
    userPublicKey: string;
    spender: string;
    amount: number;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<{
    signature: string;
    allowancePda: string;
  }> {
    // const { userPublicKey, spender } = params;

    // Get next nonce
    // const _nonce = await this.getNextAllowanceNonce({
    //   userPublicKey,
    //   spender,
    //   connection: params.connection,
    // });

    // Derive allowance PDA with nonce
    // const _allowancePda = await this.deriveAllowancePDA({
    //   userPublicKey,
    //   spender,
    //   nonce,
    // });

    // Placeholder - would implement full transaction logic from SolanaService
    throw new Error(
      "Not implemented - would use full SolanaService approveAllowanceSol logic",
    );
  }

  /**
   * Revoke an existing allowance
   */
  async revokeAllowance(_params: {
    userPublicKey: string;
    allowancePda: string;
    sendTransaction: Function;
    signTransaction: Function;
    connection?: Connection;
  }): Promise<string> {
    // Placeholder - would implement full transaction logic from SolanaService
    throw new Error(
      "Not implemented - would use full SolanaService revokeAllowance logic",
    );
  }

  /**
   * Find active allowances for a user/spender pair
   */
  async findActiveAllowances(params: {
    userPublicKey: string;
    spender: string;
    connection?: Connection;
  }): Promise<
    Array<{
      allowancePda: string;
      nonce: number;
      data: AllowanceAccountState;
    }>
  > {
    const { userPublicKey, spender, connection = this.connection } = params;

    const maxNonce = await this.getNextAllowanceNonce({
      userPublicKey,
      spender,
      connection,
    });
    const activeAllowances = [];

    // Check each possible nonce up to the current max
    for (let nonce = 0; nonce < maxNonce; nonce++) {
      try {
        const allowancePda = await this.deriveAllowancePDA({
          userPublicKey,
          spender,
          nonce,
        });

        const info = await this.getAllowanceInfo(allowancePda, connection);

        if (info.accountExists && info.allowanceData) {
          activeAllowances.push({
            allowancePda,
            nonce,
            data: info.allowanceData,
          });
        }
      } catch (error) {
        // Skip failed lookups
        continue;
      }
    }

    return activeAllowances;
  }
}

/**
 * Factory function to create an allowance service
 */
export function createAllowanceService(
  config: AtomikConfig,
): AtomikAllowanceService {
  return new AtomikAllowanceService(config);
}
