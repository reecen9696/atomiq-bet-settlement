import { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { useApi } from '../hooks/useApi';
import { Coins, Loader2, TrendingUp, TrendingDown, ExternalLink } from 'lucide-react';
import { solanaService } from '../services/solana';
import type { Bet } from '../types';

export function BettingInterface() {
  const { publicKey } = useWallet();
  const { createBet, isLoading } = useApi();
  const [amount, setAmount] = useState('0.1');
  const [choice, setChoice] = useState<'heads' | 'tails'>('heads');
  const [lastBet, setLastBet] = useState<Bet | null>(null);
  const [error, setError] = useState<string>('');

  const handlePlaceBet = async () => {
    if (!publicKey) {
      setError('Please connect your wallet first');
      return;
    }

    const amountNum = parseFloat(amount);
    if (isNaN(amountNum) || amountNum < 0.1) {
      setError('Minimum bet is 0.1 SOL');
      return;
    }

    setError('');
    try {
      const response = await createBet({
        stake_amount: Math.floor(amountNum * 1_000_000_000), // Convert to lamports
        stake_token: 'SOL',
        choice,
        user_wallet: publicKey.toBase58(),
      });

      setLastBet(response.bet);
      console.log('Bet created:', response.bet);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to place bet';
      setError(errorMsg);
      console.error('Bet placement error:', err);
    }
  };

  if (!publicKey) {
    return (
      <div className="p-6 bg-gray-50 rounded-lg border border-gray-200">
        <div className="flex items-center text-gray-500">
          <Coins className="w-5 h-5 mr-2" />
          <span>Connect wallet to place bets</span>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow-lg border border-gray-200">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-800 flex items-center">
          <Coins className="w-6 h-6 mr-2 text-orange-500" />
          Place Bet
        </h2>
      </div>

      <div className="space-y-4">
        {/* Amount Input */}
        <div>
          <label className="block text-sm font-semibold text-gray-700 mb-2">
            Bet Amount (SOL)
          </label>
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            min="0.1"
            step="0.1"
            className="w-full px-4 py-3 border-2 border-gray-300 rounded-lg focus:border-orange-500 focus:ring-2 focus:ring-orange-200 transition-all text-lg font-mono"
            placeholder="0.1"
          />
          <p className="text-xs text-gray-500 mt-1">
            Minimum: 0.1 SOL (~{(parseFloat(amount) * 1_000_000_000).toLocaleString()} lamports)
          </p>
        </div>

        {/* Choice Selection */}
        <div>
          <label className="block text-sm font-semibold text-gray-700 mb-2">
            Your Prediction
          </label>
          <div className="grid grid-cols-2 gap-3">
            <button
              onClick={() => setChoice('heads')}
              className={`p-4 rounded-lg border-2 transition-all ${
                choice === 'heads'
                  ? 'border-orange-500 bg-orange-50 shadow-md'
                  : 'border-gray-300 bg-white hover:border-orange-300'
              }`}
            >
              <div className="text-center">
                <div className="text-3xl mb-2">🪙</div>
                <div className="font-bold text-gray-800">Heads</div>
              </div>
            </button>
            <button
              onClick={() => setChoice('tails')}
              className={`p-4 rounded-lg border-2 transition-all ${
                choice === 'tails'
                  ? 'border-orange-500 bg-orange-50 shadow-md'
                  : 'border-gray-300 bg-white hover:border-orange-300'
              }`}
            >
              <div className="text-center">
                <div className="text-3xl mb-2">🎯</div>
                <div className="font-bold text-gray-800">Tails</div>
              </div>
            </button>
          </div>
        </div>

        {/* Error Display */}
        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-sm text-red-800">{error}</p>
          </div>
        )}

        {/* Place Bet Button */}
        <button
          onClick={handlePlaceBet}
          disabled={isLoading}
          className="w-full bg-gradient-to-r from-orange-500 to-red-600 text-white px-6 py-4 rounded-lg hover:from-orange-600 hover:to-red-700 disabled:from-gray-400 disabled:to-gray-500 disabled:cursor-not-allowed transition-all duration-200 font-bold text-lg shadow-lg hover:shadow-xl flex items-center justify-center"
        >
          {isLoading ? (
            <>
              <Loader2 className="w-6 h-6 animate-spin mr-2" />
              Placing Bet...
            </>
          ) : (
            `Place ${amount} SOL Bet`
          )}
        </button>

        {/* Last Bet Result */}
        {lastBet && (
          <div className={`p-4 rounded-lg border-2 ${
            lastBet.won === true
              ? 'bg-green-50 border-green-300'
              : lastBet.won === false
              ? 'bg-red-50 border-red-300'
              : 'bg-blue-50 border-blue-300'
          }`}>
            <div className="flex items-start justify-between mb-2">
              <h3 className="font-bold text-gray-800 flex items-center">
                {lastBet.won === true && <TrendingUp className="w-5 h-5 mr-1 text-green-600" />}
                {lastBet.won === false && <TrendingDown className="w-5 h-5 mr-1 text-red-600" />}
                Last Bet
              </h3>
              <span className={`px-2 py-1 rounded text-xs font-bold ${getStatusBadgeClass(lastBet.status)}`}>
                {lastBet.status}
              </span>
            </div>

            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Bet ID:</span>
                <code className="text-xs font-mono">{lastBet.bet_id.slice(0, 8)}...</code>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Amount:</span>
                <span className="font-semibold">
                  {(lastBet.stake_amount / 1_000_000_000).toFixed(3)} SOL
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Choice:</span>
                <span className="font-semibold uppercase">{lastBet.choice}</span>
              </div>
              {lastBet.won !== null && (
                <div className="flex justify-between">
                  <span className="text-gray-600">Result:</span>
                  <span className={`font-bold ${lastBet.won ? 'text-green-600' : 'text-red-600'}`}>
                    {lastBet.won ? 'WON! 🎉' : 'LOST 😔'}
                  </span>
                </div>
              )}
              {lastBet.payout_amount && (
                <div className="flex justify-between">
                  <span className="text-gray-600">Payout:</span>
                  <span className="font-semibold text-green-600">
                    {(lastBet.payout_amount / 1_000_000_000).toFixed(3)} SOL
                  </span>
                </div>
              )}
              {lastBet.solana_tx_id && (
                <div className="pt-2 border-t border-gray-300">
                  <p className="text-xs text-gray-600 mb-1">Transaction ID:</p>
                  <div className="flex items-center justify-between bg-white p-2 rounded">
                    <code className="text-xs font-mono truncate mr-2">
                      {lastBet.solana_tx_id}
                    </code>
                    <a
                      href={solanaService.getExplorerUrl(lastBet.solana_tx_id)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-600 hover:text-blue-800 flex-shrink-0"
                    >
                      <ExternalLink className="w-4 h-4" />
                    </a>
                  </div>
                </div>
              )}
              {lastBet.last_error_message && (
                <div className="pt-2 border-t border-red-200">
                  <p className="text-xs text-red-600">
                    Error: {lastBet.last_error_message}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function getStatusBadgeClass(status: string): string {
  const normalized = status.toLowerCase();
  switch (normalized) {
    case 'completed':
      return 'bg-green-100 text-green-800';
    case 'pending':
      return 'bg-yellow-100 text-yellow-800';
    case 'batched':
    case 'submitted_to_solana':
    case 'confirmed_on_solana':
      return 'bg-blue-100 text-blue-800';
    case 'failed_retryable':
      return 'bg-orange-100 text-orange-800';
    case 'failed_manual_review':
      return 'bg-red-100 text-red-800';
    default:
      return 'bg-gray-100 text-gray-800';
  }
}