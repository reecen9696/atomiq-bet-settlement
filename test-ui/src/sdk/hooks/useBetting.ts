import { useState, useCallback, useEffect } from "react";
import type {
  AtomikBettingService,
  CoinflipResult,
  Settlement,
  RecentGame,
} from "../index";

export interface UseBettingState {
  // Current game state
  currentGame: {
    gameId: string;
    choice: "heads" | "tails";
    amount: number;
  } | null;

  // Game result
  gameResult: CoinflipResult | null;

  // Recent games
  recentGames: RecentGame[];
  gamesCursor: string | null;
  hasMoreGames: boolean;

  // Pending settlements
  pendingSettlements: Settlement[];

  // Loading states
  loading: boolean;
  placingBet: boolean;
  loadingGames: boolean;
  loadingSettlements: boolean;

  // Error state
  error: string | null;
}

export interface UseBettingActions {
  // Core betting operations
  placeCoinflipBet: (
    choice: "heads" | "tails",
    amount: number,
    vaultPda?: string,
    allowancePda?: string,
  ) => Promise<CoinflipResult | null>;

  // Game result operations
  checkGameResult: (gameId: string) => Promise<CoinflipResult | null>;
  waitForGameResult: (
    gameId: string,
    timeoutMs?: number,
  ) => Promise<CoinflipResult | null>;

  // History and settlements
  loadRecentGames: (cursor?: string) => Promise<void>;
  loadMoreGames: () => Promise<void>;
  refreshPendingSettlements: () => Promise<void>;

  // State management
  clearCurrentGame: () => void;
  clearError: () => void;
  reset: () => void;
}

export interface UseBettingResult extends UseBettingState, UseBettingActions {}

/**
 * React hook for betting operations
 */
export function useBetting(
  userPublicKey: string | null,
  bettingService: AtomikBettingService,
): UseBettingResult {
  const [state, setState] = useState<UseBettingState>({
    currentGame: null,
    gameResult: null,
    recentGames: [],
    gamesCursor: null,
    hasMoreGames: true,
    pendingSettlements: [],
    loading: false,
    placingBet: false,
    loadingGames: false,
    loadingSettlements: false,
    error: null,
  });

  // Auto-load data when user changes
  useEffect(() => {
    if (userPublicKey) {
      loadRecentGames();
      refreshPendingSettlements();
    } else {
      reset();
    }
  }, [userPublicKey]);

  const placeCoinflipBet = useCallback(
    async (
      choice: "heads" | "tails",
      amount: number,
      vaultPda?: string,
      allowancePda?: string,
    ): Promise<CoinflipResult | null> => {
      if (!userPublicKey) {
        setState((prev) => ({ ...prev, error: "No user connected" }));
        return null;
      }

      if (amount <= 0) {
        setState((prev) => ({
          ...prev,
          error: "Bet amount must be greater than 0",
        }));
        return null;
      }

      setState((prev) => ({
        ...prev,
        placingBet: true,
        error: null,
        currentGame: { gameId: "", choice, amount },
        gameResult: null,
      }));

      try {
        const result = await bettingService.placeCoinflipBet({
          userPublicKey,
          choice,
          amount,
          vaultPda,
          allowancePda,
        });

        setState((prev) => ({
          ...prev,
          placingBet: false,
          currentGame: { gameId: result.gameId, choice, amount },
          gameResult: result,
        }));

        // Refresh recent games and settlements after bet
        await Promise.all([loadRecentGames(), refreshPendingSettlements()]);

        return result;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to place bet",
          placingBet: false,
          currentGame: null,
        }));
        return null;
      }
    },
    [userPublicKey, bettingService],
  );

  const checkGameResult = useCallback(
    async (gameId: string): Promise<CoinflipResult | null> => {
      setState((prev) => ({ ...prev, loading: true, error: null }));

      try {
        const result = await bettingService.getGameResult(gameId);

        setState((prev) => ({
          ...prev,
          loading: false,
          gameResult: result,
        }));

        return result;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to check game result",
          loading: false,
        }));
        return null;
      }
    },
    [bettingService],
  );

  const waitForGameResult = useCallback(
    async (
      gameId: string,
      timeoutMs = 30000,
    ): Promise<CoinflipResult | null> => {
      setState((prev) => ({ ...prev, loading: true, error: null }));

      try {
        const result = await bettingService.waitForGameSettlement(
          gameId,
          timeoutMs,
        );

        setState((prev) => ({
          ...prev,
          loading: false,
          gameResult: result,
        }));

        // Refresh settlements after game settles
        await refreshPendingSettlements();

        return result;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Game settlement timeout",
          loading: false,
        }));
        return null;
      }
    },
    [bettingService],
  );

  const loadRecentGames = useCallback(
    async (cursor?: string) => {
      setState((prev) => ({ ...prev, loadingGames: true, error: null }));

      try {
        const result = await bettingService.getRecentGames(cursor);

        setState((prev) => ({
          ...prev,
          recentGames: cursor
            ? [...prev.recentGames, ...result.games]
            : result.games,
          gamesCursor: result.cursor || null,
          hasMoreGames: result.hasMore,
          loadingGames: false,
        }));
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: (error as Error).message || "Failed to load recent games",
          loadingGames: false,
        }));
      }
    },
    [bettingService],
  );

  const loadMoreGames = useCallback(async () => {
    if (state.gamesCursor && state.hasMoreGames && !state.loadingGames) {
      await loadRecentGames(state.gamesCursor);
    }
  }, [
    state.gamesCursor,
    state.hasMoreGames,
    state.loadingGames,
    loadRecentGames,
  ]);

  const refreshPendingSettlements = useCallback(async () => {
    if (!userPublicKey) return;

    setState((prev) => ({ ...prev, loadingSettlements: true, error: null }));

    try {
      const settlements =
        await bettingService.getPendingSettlements(userPublicKey);

      setState((prev) => ({
        ...prev,
        pendingSettlements: settlements,
        loadingSettlements: false,
      }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        error: (error as Error).message || "Failed to load pending settlements",
        loadingSettlements: false,
      }));
    }
  }, [userPublicKey, bettingService]);

  const clearCurrentGame = useCallback(() => {
    setState((prev) => ({
      ...prev,
      currentGame: null,
      gameResult: null,
    }));
  }, []);

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  const reset = useCallback(() => {
    setState({
      currentGame: null,
      gameResult: null,
      recentGames: [],
      gamesCursor: null,
      hasMoreGames: true,
      pendingSettlements: [],
      loading: false,
      placingBet: false,
      loadingGames: false,
      loadingSettlements: false,
      error: null,
    });
  }, []);

  return {
    ...state,
    placeCoinflipBet,
    checkGameResult,
    waitForGameResult,
    loadRecentGames,
    loadMoreGames,
    refreshPendingSettlements,
    clearCurrentGame,
    clearError,
    reset,
  };
}

/**
 * Hook for just checking game results without full betting state
 */
export function useGameResult(
  gameId: string | null,
  bettingService: AtomikBettingService,
) {
  const [result, setResult] = useState<CoinflipResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkResult = useCallback(async () => {
    if (!gameId) return;

    setLoading(true);
    setError(null);

    try {
      const gameResult = await bettingService.getGameResult(gameId);
      setResult(gameResult);
    } catch (err) {
      setError((err as Error).message || "Failed to check game result");
    } finally {
      setLoading(false);
    }
  }, [gameId, bettingService]);

  useEffect(() => {
    checkResult();
  }, [checkResult]);

  return {
    result,
    loading,
    error,
    refresh: checkResult,
  };
}
