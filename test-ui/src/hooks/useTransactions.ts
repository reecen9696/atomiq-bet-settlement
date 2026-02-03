import { useState, useEffect, useCallback } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { useApi } from "./useApi";
import type { SettlementGame, SettlementGameDetail } from "../types";

type TransactionRow = (SettlementGame | SettlementGameDetail) & {
  _lastSeenAtMs: number;
  _isPending: boolean;
};

export function useTransactions() {
  const { publicKey } = useWallet();
  const [transactions, setTransactions] = useState<TransactionRow[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { getPendingSettlements, getSettlementGame } = useApi();

  const pruneOld = useCallback((rows: TransactionRow[]): TransactionRow[] => {
    const cutoffMs = Date.now() - 10 * 60 * 1000; // keep 10 minutes
    return rows.filter((r) => r._lastSeenAtMs >= cutoffMs);
  }, []);

  const fetchTransactions = useCallback(async () => {
    try {
      setError(null);
      const resp = await getPendingSettlements();
      const games = Array.isArray(resp.games) ? resp.games : [];
      // Sort newest first by transaction_id (monotonic-ish)
      games.sort((a, b) => (b.transaction_id ?? 0) - (a.transaction_id ?? 0));

      const now = Date.now();
      const pendingIds = new Set<number>(games.map((g) => g.transaction_id));

      // Merge into existing list so items don't disappear when they leave the pending list.
      setTransactions((prev) => {
        const nextById = new Map<number, TransactionRow>();

        for (const row of prev) {
          nextById.set(row.transaction_id, row);
        }

        for (const g of games) {
          const existing = nextById.get(g.transaction_id);
          nextById.set(g.transaction_id, {
            ...(existing ?? g),
            ...g,
            _isPending: true,
            _lastSeenAtMs: now,
          });
        }

        // Mark rows that disappeared from pending as no longer pending
        for (const [txId, row] of nextById) {
          if (!pendingIds.has(txId)) {
            nextById.set(txId, {
              ...row,
              _isPending: false,
            });
          }
        }

        const merged = Array.from(nextById.values());
        merged.sort(
          (a, b) => (b.transaction_id ?? 0) - (a.transaction_id ?? 0),
        );
        return pruneOld(merged);
      });

      // For rows that have left pending, fetch their final settlement status.
      // Do this after updating state; fetches are best-effort.
      for (const row of games) {
        // no-op: still pending
        void row;
      }

      // Kick off a best-effort enrichment for any non-pending rows we already have.
      setTransactions((prev) => {
        for (const row of prev) {
          if (row._isPending) continue;
          if ("settlement_status" in row) continue;
          // Fetch detail asynchronously (no await here)
          getSettlementGame(row.transaction_id)
            .then((detail) => {
              setTransactions((curr) => {
                const updated = curr.map((r) =>
                  r.transaction_id === detail.transaction_id
                    ? {
                        ...(r as TransactionRow),
                        ...detail,
                        _isPending: false,
                        _lastSeenAtMs: Date.now(),
                      }
                    : r,
                );
                return pruneOld(updated);
              });
            })
            .catch(() => {
              // ignore
            });
        }
        return prev;
      });
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : "Failed to fetch pending settlements";
      console.error("Failed to fetch transactions:", error);
      setError(message);
      setTransactions([]);
    } finally {
      setIsLoading(false);
    }
  }, [getPendingSettlements, getSettlementGame, pruneOld]);

  useEffect(() => {
    fetchTransactions();
    // Manual refresh only - no automatic polling
  }, [fetchTransactions]);

  return {
    transactions,
    isLoading,
    refresh: fetchTransactions,
    currentWallet: publicKey?.toBase58() || null,
    error,
  };
}
