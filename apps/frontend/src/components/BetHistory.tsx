'use client';

import { BetStatus } from '@atomik/types';

export function BetHistory() {
  // Mock data - replace with actual API query
  const bets: any[] = [];

  const getStatusColor = (status: BetStatus) => {
    switch (status) {
      case BetStatus.Completed:
        return 'text-green-400';
      case BetStatus.Pending:
      case BetStatus.Batched:
      case BetStatus.SubmittedToSolana:
        return 'text-yellow-400';
      case BetStatus.FailedRetryable:
      case BetStatus.FailedManualReview:
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  return (
    <div className="bg-gray-900 rounded-lg p-6">
      <h2 className="text-2xl font-bold mb-6">Bet History</h2>

      {bets.length === 0 ? (
        <div className="text-center text-gray-400 py-12">
          No bets yet. Place your first bet above!
        </div>
      ) : (
        <div className="space-y-3">
          {bets.map((bet) => (
            <div key={bet.bet_id} className="bg-gray-800 rounded-lg p-4">
              <div className="flex justify-between items-start mb-2">
                <div>
                  <div className="font-semibold">
                    {bet.stake_amount / 1e9} SOL - {bet.choice}
                  </div>
                  <div className="text-xs text-gray-400">
                    {new Date(bet.created_at).toLocaleString()}
                  </div>
                </div>
                <div className={`text-sm font-semibold ${getStatusColor(bet.status)}`}>
                  {bet.status.replace(/_/g, ' ').toUpperCase()}
                </div>
              </div>

              {bet.won !== null && (
                <div className="mt-2 pt-2 border-t border-gray-700">
                  <div className="flex justify-between text-sm">
                    <span>Result:</span>
                    <span className={bet.won ? 'text-green-400' : 'text-red-400'}>
                      {bet.won ? '✓ Won' : '✗ Lost'}
                    </span>
                  </div>
                  {bet.payout_amount > 0 && (
                    <div className="flex justify-between text-sm mt-1">
                      <span>Payout:</span>
                      <span className="text-green-400">
                        {bet.payout_amount / 1e9} SOL
                      </span>
                    </div>
                  )}
                </div>
              )}

              {bet.solana_tx_id && (
                <div className="mt-2 text-xs">
                  <a
                    href={`https://explorer.solana.com/tx/${bet.solana_tx_id}?cluster=devnet`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-400 hover:text-blue-300"
                  >
                    View on Explorer →
                  </a>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
