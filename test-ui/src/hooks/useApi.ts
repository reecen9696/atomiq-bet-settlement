import { useState, useCallback } from "react";
import { apiService } from "../services/api";
import type {
  CoinFlipPlayRequest,
  GameResponse,
  PendingSettlementsResponse,
  RecentGamesResponse,
  SettlementGameDetail,
} from "../types";

export function useApi() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const playCoinflip = useCallback(
    async (request: CoinFlipPlayRequest): Promise<GameResponse> => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await apiService.playCoinflip(request);
        return response;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "Failed to play coinflip";
        setError(errorMessage);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [],
  );

  const getPendingSettlements =
    useCallback(async (): Promise<PendingSettlementsResponse> => {
      setIsLoading(true);
      setError(null);
      try {
        return await apiService.getPendingSettlements({ limit: 100 });
      } catch (err) {
        const errorMessage =
          err instanceof Error
            ? err.message
            : "Failed to fetch pending settlements";
        setError(errorMessage);
        return { games: [], next_cursor: null };
      } finally {
        setIsLoading(false);
      }
    }, []);

  const getGameResult = useCallback(
    async (gameId: string): Promise<GameResponse> => {
      setIsLoading(true);
      setError(null);
      try {
        return await apiService.getGameResult(gameId);
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "Failed to fetch game result";
        setError(errorMessage);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [],
  );

  const getSettlementGame = useCallback(
    async (txId: number): Promise<SettlementGameDetail> => {
      setIsLoading(true);
      setError(null);
      try {
        return await apiService.getSettlementGame(txId);
      } catch (err) {
        const errorMessage =
          err instanceof Error
            ? err.message
            : "Failed to fetch settlement game";
        setError(errorMessage);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [],
  );

  const getRecentGames = useCallback(async (): Promise<RecentGamesResponse> => {
    setIsLoading(true);
    setError(null);
    try {
      return await apiService.getRecentGames({ limit: 50 });
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to fetch recent games";
      setError(errorMessage);
      return { games: [], next_cursor: null };
    } finally {
      setIsLoading(false);
    }
  }, []);

  const checkHealth = useCallback(async () => {
    try {
      const health = await apiService.healthCheck();
      return health;
    } catch (err) {
      console.error("Health check failed:", err);
      return null;
    }
  }, []);

  return {
    playCoinflip,
    getPendingSettlements,
    getGameResult,
    getSettlementGame,
    getRecentGames,
    checkHealth,
    isLoading,
    error,
  };
}
