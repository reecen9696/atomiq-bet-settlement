import { useState, useEffect, useCallback, useRef } from "react";
import type {
  AtomikVaultService,
} from "../index";
import type { VaultAccountState } from "../../services/solana/types";

export interface UseVaultState {
  // Vault info
  vaultPda: string | null;
  vaultExists: boolean;
  vaultData: VaultAccountState | null;
  balance: number | null;

  // Loading states
  loading: boolean;
  initializing: boolean;
  depositing: boolean;
  withdrawing: boolean;

  // Error state
  error: string | null;

  // Last operation result
  lastSignature: string | null;
}

export interface UseVaultActions {
  // Core operations
  initialize: () => Promise<string | null>;
  deposit: (amount: number) => Promise<string | null>;
  withdraw: (amount: number) => Promise<string | null>;

  // Utility operations
  refreshVaultInfo: () => Promise<void>;
  refreshBalance: () => Promise<void>;
  requestAirdrop: (amount?: number) => Promise<string | null>;

  // State management
  clearError: () => void;
  reset: () => void;
}

export interface UseVaultResult extends UseVaultState, UseVaultActions {}

/**
 * React hook for vault operations
 */
export function useVault(
  userPublicKey: string | null,
  vaultService: AtomikVaultService,
  sendTransaction?: Function,
  signTransaction?: Function,
): UseVaultResult {
  const [state, setState] = useState<UseVaultState>({
    vaultPda: null,
    vaultExists: false,
    vaultData: null,
    balance: null,
    loading: false,
    initializing: false,
    depositing: false,
    withdrawing: false,
    error: null,
    lastSignature: null,
  });

  const abortControllerRef = useRef<AbortController | null>(null);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, []);

  // Update vault PDA when user changes
  useEffect(() => {
    if (userPublicKey) {
      vaultService.deriveVaultPDA(userPublicKey).then((vaultPda) => {
        setState((prev) => ({ ...prev, vaultPda }));
      });
    } else {
      setState((prev) => ({
        ...prev,
        vaultPda: null,
        vaultData: null,
        vaultExists: false,
      }));
    }
  }, [userPublicKey, vaultService]);

  // Auto-refresh vault info when vault PDA is available
  useEffect(() => {
    if (userPublicKey && state.vaultPda) {
      refreshVaultInfo();
      refreshBalance();
    }
  }, [userPublicKey, state.vaultPda]);

  const refreshVaultInfo = useCallback(async () => {
    if (!userPublicKey) return;

    setState((prev) => ({ ...prev, loading: true, error: null }));

    try {
      const vaultInfo = await vaultService.getVaultInfo({ userPublicKey });

      setState((prev) => ({
        ...prev,
        vaultPda: vaultInfo.vaultPda,
        vaultExists: vaultInfo.accountExists,
        vaultData: vaultInfo.vaultData,
        loading: false,
      }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        error: (error as Error).message || "Failed to fetch vault info",
        loading: false,
      }));
    }
  }, [userPublicKey, vaultService]);

  const refreshBalance = useCallback(async () => {
    if (!userPublicKey) return;

    try {
      const balance = await vaultService.getBalance(userPublicKey);
      setState((prev) => ({ ...prev, balance }));
    } catch (error) {
      console.warn("Failed to fetch balance:", error);
    }
  }, [userPublicKey, vaultService]);

  const initialize = useCallback(async (): Promise<string | null> => {
    if (!userPublicKey || !sendTransaction || !signTransaction) {
      setState((prev) => ({
        ...prev,
        error: "Missing required parameters for initialization",
      }));
      return null;
    }

    setState((prev) => ({ ...prev, initializing: true, error: null }));

    try {
      const signature = await vaultService.initializeUserVault({
        userPublicKey,
        sendTransaction,
        signTransaction,
      });

      setState((prev) => ({
        ...prev,
        initializing: false,
        lastSignature: signature,
      }));

      // Refresh vault info after initialization
      await refreshVaultInfo();

      return signature;
    } catch (error) {
      setState((prev) => ({
        ...prev,
        error: (error as Error).message || "Failed to initialize vault",
        initializing: false,
      }));
      return null;
    }
  }, [
    userPublicKey,
    sendTransaction,
    signTransaction,
    vaultService,
    refreshVaultInfo,
  ]);

  const deposit = useCallback(
    async (amount: number): Promise<string | null> => {
      if (!userPublicKey || !sendTransaction || !signTransaction) {
        setState((prev) => ({
          ...prev,
          error: "Missing required parameters for deposit",
        }));
        return null;
      }

      if (amount <= 0) {
        setState((prev) => ({
          ...prev,
          error: "Deposit amount must be greater than 0",
        }));
        return null;
      }

      setState((prev) => ({ ...prev, depositing: true, error: null }));

      try {
        const signature = await vaultService.depositSol({
          userPublicKey,
          amount,
          sendTransaction,
          signTransaction,
        });

        setState((prev) => ({
          ...prev,
          depositing: false,
          lastSignature: signature,
        }));

        // Refresh vault info and balance after deposit
        await Promise.all([refreshVaultInfo(), refreshBalance()]);

        return signature;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to deposit SOL",
          depositing: false,
        }));
        return null;
      }
    },
    [
      userPublicKey,
      sendTransaction,
      signTransaction,
      vaultService,
      refreshVaultInfo,
      refreshBalance,
    ],
  );

  const withdraw = useCallback(
    async (amount: number): Promise<string | null> => {
      if (!userPublicKey || !sendTransaction || !signTransaction) {
        setState((prev) => ({
          ...prev,
          error: "Missing required parameters for withdrawal",
        }));
        return null;
      }

      if (amount <= 0) {
        setState((prev) => ({
          ...prev,
          error: "Withdrawal amount must be greater than 0",
        }));
        return null;
      }

      setState((prev) => ({ ...prev, withdrawing: true, error: null }));

      try {
        const signature = await vaultService.withdrawSol({
          userPublicKey,
          amount,
          sendTransaction,
          signTransaction,
        });

        setState((prev) => ({
          ...prev,
          withdrawing: false,
          lastSignature: signature,
        }));

        // Refresh vault info and balance after withdrawal
        await Promise.all([refreshVaultInfo(), refreshBalance()]);

        return signature;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to withdraw SOL",
          withdrawing: false,
        }));
        return null;
      }
    },
    [
      userPublicKey,
      sendTransaction,
      signTransaction,
      vaultService,
      refreshVaultInfo,
      refreshBalance,
    ],
  );

  const requestAirdrop = useCallback(
    async (amount = 1): Promise<string | null> => {
      if (!userPublicKey) {
        setState((prev) => ({ ...prev, error: "No user public key provided" }));
        return null;
      }

      setState((prev) => ({ ...prev, loading: true, error: null }));

      try {
        const signature = await vaultService.requestAirdrop(
          userPublicKey,
          amount,
        );

        setState((prev) => ({
          ...prev,
          loading: false,
          lastSignature: signature,
        }));

        // Refresh balance after airdrop
        setTimeout(() => refreshBalance(), 3000); // Give it time to confirm

        return signature;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to request airdrop",
          loading: false,
        }));
        return null;
      }
    },
    [userPublicKey, vaultService, refreshBalance],
  );

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  const reset = useCallback(() => {
    setState({
      vaultPda: null,
      vaultExists: false,
      vaultData: null,
      balance: null,
      loading: false,
      initializing: false,
      depositing: false,
      withdrawing: false,
      error: null,
      lastSignature: null,
    });
  }, []);

  return {
    ...state,
    initialize,
    deposit,
    withdraw,
    refreshVaultInfo,
    refreshBalance,
    requestAirdrop,
    clearError,
    reset,
  };
}
