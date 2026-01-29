import { useState, useCallback, useRef, useEffect } from "react";
import type { AtomikAllowanceService } from "../index";
import type { AllowanceAccountState } from "../../services/solana/types";

export interface UseAllowanceState {
  // Current allowances
  activeAllowances: Array<{
    allowancePda: string;
    nonce: number;
    data: AllowanceAccountState;
  }>;

  // Loading states
  loading: boolean;
  approving: boolean;
  revoking: boolean;

  // Error state
  error: string | null;

  // Last operation result
  lastSignature: string | null;
  lastAllowancePda: string | null;
}

export interface UseAllowanceActions {
  // Core operations
  approve: (
    spender: string,
    amount: number,
  ) => Promise<{ signature: string; allowancePda: string } | null>;
  revoke: (allowancePda: string) => Promise<string | null>;

  // Utility operations
  refreshAllowances: (spender: string) => Promise<void>;
  getNextNonce: (spender: string) => Promise<number>;

  // State management
  clearError: () => void;
  reset: () => void;
}

export interface UseAllowanceResult
  extends UseAllowanceState, UseAllowanceActions {}

/**
 * React hook for allowance operations
 */
export function useAllowance(
  userPublicKey: string | null,
  allowanceService: AtomikAllowanceService,
  sendTransaction?: Function,
  signTransaction?: Function,
): UseAllowanceResult {
  const [state, setState] = useState<UseAllowanceState>({
    activeAllowances: [],
    loading: false,
    approving: false,
    revoking: false,
    error: null,
    lastSignature: null,
    lastAllowancePda: null,
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

  const refreshAllowances = useCallback(
    async (spender: string) => {
      if (!userPublicKey) return;

      setState((prev) => ({ ...prev, loading: true, error: null }));

      try {
        const allowances = await allowanceService.findActiveAllowances({
          userPublicKey,
          spender,
        });

        setState((prev) => ({
          ...prev,
          activeAllowances: allowances,
          loading: false,
        }));
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to fetch allowances",
          loading: false,
        }));
      }
    },
    [userPublicKey, allowanceService],
  );

  const getNextNonce = useCallback(
    async (spender: string): Promise<number> => {
      if (!userPublicKey) return 0;

      try {
        return await allowanceService.getNextAllowanceNonce({
          userPublicKey,
          spender,
        });
      } catch (error) {
        console.warn("Failed to get next nonce:", error);
        return 0;
      }
    },
    [userPublicKey, allowanceService],
  );

  const approve = useCallback(
    async (
      spender: string,
      amount: number,
    ): Promise<{ signature: string; allowancePda: string } | null> => {
      if (!userPublicKey || !sendTransaction || !signTransaction) {
        setState((prev) => ({
          ...prev,
          error: "Missing required parameters for approval",
        }));
        return null;
      }

      if (amount <= 0) {
        setState((prev) => ({
          ...prev,
          error: "Allowance amount must be greater than 0",
        }));
        return null;
      }

      setState((prev) => ({ ...prev, approving: true, error: null }));

      try {
        const result = await allowanceService.approveAllowance({
          userPublicKey,
          spender,
          amount,
          sendTransaction,
          signTransaction,
        });

        setState((prev) => ({
          ...prev,
          approving: false,
          lastSignature: result.signature,
          lastAllowancePda: result.allowancePda,
        }));

        // Refresh allowances after approval
        await refreshAllowances(spender);

        return result;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to approve allowance",
          approving: false,
        }));
        return null;
      }
    },
    [
      userPublicKey,
      sendTransaction,
      signTransaction,
      allowanceService,
      refreshAllowances,
    ],
  );

  const revoke = useCallback(
    async (allowancePda: string): Promise<string | null> => {
      if (!userPublicKey || !sendTransaction || !signTransaction) {
        setState((prev) => ({
          ...prev,
          error: "Missing required parameters for revocation",
        }));
        return null;
      }

      setState((prev) => ({ ...prev, revoking: true, error: null }));

      try {
        const signature = await allowanceService.revokeAllowance({
          userPublicKey,
          allowancePda,
          sendTransaction,
          signTransaction,
        });

        setState((prev) => ({
          ...prev,
          revoking: false,
          lastSignature: signature,
          lastAllowancePda: allowancePda,
          // Remove revoked allowance from active list
          activeAllowances: prev.activeAllowances.filter(
            (a) => a.allowancePda !== allowancePda,
          ),
        }));

        return signature;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to revoke allowance",
          revoking: false,
        }));
        return null;
      }
    },
    [userPublicKey, sendTransaction, signTransaction, allowanceService],
  );

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  const reset = useCallback(() => {
    setState({
      activeAllowances: [],
      loading: false,
      approving: false,
      revoking: false,
      error: null,
      lastSignature: null,
      lastAllowancePda: null,
    });
  }, []);

  return {
    ...state,
    approve,
    revoke,
    refreshAllowances,
    getNextNonce,
    clearError,
    reset,
  };
}

/**
 * Hook for managing a specific allowance (spender-specific)
 */
export function useAllowanceForSpender(
  userPublicKey: string | null,
  spender: string,
  allowanceService: AtomikAllowanceService,
  sendTransaction?: Function,
  signTransaction?: Function,
) {
  const allowanceHook = useAllowance(
    userPublicKey,
    allowanceService,
    sendTransaction,
    signTransaction,
  );

  // Auto-refresh allowances for this spender when user changes
  useEffect(() => {
    if (userPublicKey && spender) {
      allowanceHook.refreshAllowances(spender);
    }
  }, [userPublicKey, spender]);

  // Filter allowances for this spender only
  const spenderAllowances = allowanceHook.activeAllowances;

  // Convenience methods for this specific spender
  const approveForSpender = useCallback(
    (amount: number) => allowanceHook.approve(spender, amount),
    [allowanceHook.approve, spender],
  );

  const refreshForSpender = useCallback(
    () => allowanceHook.refreshAllowances(spender),
    [allowanceHook.refreshAllowances, spender],
  );

  const getNextNonceForSpender = useCallback(
    () => allowanceHook.getNextNonce(spender),
    [allowanceHook.getNextNonce, spender],
  );

  return {
    ...allowanceHook,
    allowances: spenderAllowances,
    approve: approveForSpender,
    refresh: refreshForSpender,
    getNextNonce: getNextNonceForSpender,
  };
}
