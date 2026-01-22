import { useCallback, useEffect, useState } from "react";
import { useApi } from "./useApi";
import type { RecentGameSummary } from "../types";

export function useRecentGames() {
  const { getRecentGames } = useApi();
  const [games, setGames] = useState<RecentGameSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setError(null);
      const resp = await getRecentGames();
      const next = Array.isArray(resp.games) ? resp.games : [];
      next.sort((a, b) => (b.tx_id ?? 0) - (a.tx_id ?? 0));
      setGames(next);
    } catch (e) {
      const msg =
        e instanceof Error ? e.message : "Failed to fetch recent games";
      setError(msg);
      setGames([]);
    } finally {
      setIsLoading(false);
    }
  }, [getRecentGames]);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 5000);
    return () => clearInterval(interval);
  }, [refresh]);

  return { games, isLoading, error, refresh };
}
