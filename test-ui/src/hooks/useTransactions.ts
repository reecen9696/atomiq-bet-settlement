import { useState, useEffect, useCallback } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { useApi } from './useApi';
import type { Bet } from '../types';

export function useTransactions() {
  const { publicKey } = useWallet();
  const [transactions, setTransactions] = useState<Bet[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const { listUserBets } = useApi();

  const fetchTransactions = useCallback(async () => {
    try {
      if (!publicKey) {
        setTransactions([]);
        return;
      }

      const bets = await listUserBets(publicKey.toBase58());
      setTransactions(Array.isArray(bets) ? bets : []);
    } catch (error) {
      console.error('Failed to fetch transactions:', error);
      setTransactions([]);
    } finally {
      setIsLoading(false);
    }
  }, [listUserBets, publicKey]);

  useEffect(() => {
    fetchTransactions();
    
    // Poll for updates every 5 seconds
    const interval = setInterval(fetchTransactions, 5000);
    
    return () => clearInterval(interval);
  }, [fetchTransactions]);

  return {
    transactions,
    isLoading,
    refresh: fetchTransactions,
  };
}