import { useState, useCallback } from 'react';
import { apiService } from '../services/api';
import type { CreateBetRequest, CreateBetResponse, Bet } from '../types';

export function useApi() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createBet = useCallback(async (request: CreateBetRequest): Promise<CreateBetResponse> => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await apiService.createBet(request);
      return response;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create bet';
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const getPendingBets = useCallback(async (): Promise<Bet[]> => {
    setIsLoading(true);
    setError(null);
    try {
      const bets = await apiService.getPendingBets();
      return bets;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch bets';
      setError(errorMessage);
      return [];
    } finally {
      setIsLoading(false);
    }
  }, []);

  const listUserBets = useCallback(async (userWallet: string): Promise<Bet[]> => {
    setIsLoading(true);
    setError(null);
    try {
      return await apiService.listUserBets(userWallet);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch user bets';
      setError(errorMessage);
      return [];
    } finally {
      setIsLoading(false);
    }
  }, []);

  const checkHealth = useCallback(async () => {
    try {
      const health = await apiService.healthCheck();
      return health;
    } catch (err) {
      console.error('Health check failed:', err);
      return null;
    }
  }, []);

  return {
    createBet,
    getPendingBets,
    listUserBets,
    checkHealth,
    isLoading,
    error,
  };
}