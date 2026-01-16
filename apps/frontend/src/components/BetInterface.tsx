'use client';

import { useState } from 'react';
import type { CreateBetRequest } from '@atomik/types';

export function BetInterface() {
  const [amount, setAmount] = useState('0.1');
  const [choice, setChoice] = useState<'heads' | 'tails'>('heads');
  const [isPlacing, setIsPlacing] = useState(false);

  const handlePlaceBet = async () => {
    setIsPlacing(true);
    try {
      const bet: CreateBetRequest = {
        stake_amount: parseFloat(amount) * 1e9, // Convert SOL to lamports
        stake_token: 'SOL',
        choice,
      };

      // TODO: Call API to place bet
      console.log('Placing bet:', bet);

      // Reset form
      setAmount('0.1');
    } catch (error) {
      console.error('Failed to place bet:', error);
    } finally {
      setIsPlacing(false);
    }
  };

  return (
    <div className="bg-gray-900 rounded-lg p-6">
      <h2 className="text-2xl font-bold mb-6">Place Bet</h2>

      <div className="space-y-6">
        <div>
          <label className="block text-sm font-medium mb-2">Game</label>
          <div className="bg-gray-800 rounded-lg p-3 text-gray-400">Coinflip</div>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Amount (SOL)</label>
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            min="0.1"
            step="0.1"
            className="w-full bg-gray-800 rounded-lg p-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <div className="text-xs text-gray-400 mt-1">Min: 0.1 SOL | Max: 1000 SOL</div>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Your Choice</label>
          <div className="grid grid-cols-2 gap-3">
            <button
              onClick={() => setChoice('heads')}
              className={`py-3 rounded-lg font-semibold transition-colors ${
                choice === 'heads'
                  ? 'bg-blue-600 hover:bg-blue-700'
                  : 'bg-gray-800 hover:bg-gray-700'
              }`}
            >
              🪙 Heads
            </button>
            <button
              onClick={() => setChoice('tails')}
              className={`py-3 rounded-lg font-semibold transition-colors ${
                choice === 'tails'
                  ? 'bg-blue-600 hover:bg-blue-700'
                  : 'bg-gray-800 hover:bg-gray-700'
              }`}
            >
              🔄 Tails
            </button>
          </div>
        </div>

        <button
          onClick={handlePlaceBet}
          disabled={isPlacing}
          className="w-full py-3 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg font-semibold transition-colors"
        >
          {isPlacing ? 'Placing Bet...' : 'Place Bet'}
        </button>

        <div className="text-xs text-gray-400 text-center">
          No wallet signature needed - uses allowance
        </div>
      </div>
    </div>
  );
}
