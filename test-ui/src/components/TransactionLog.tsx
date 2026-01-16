import { useTransactions } from '../hooks/useTransactions';
import { History, Loader2, ExternalLink, RefreshCw } from 'lucide-react';
import { solanaService } from '../services/solana';
import type { Bet, BetStatus } from '../types';

export function TransactionLog() {
  const { transactions, isLoading, refresh } = useTransactions();

  if (isLoading && transactions.length === 0) {
    return (
      <div className="p-6 bg-white rounded-lg shadow-lg border border-gray-200">
        <div className="flex items-center justify-center">
          <Loader2 className="w-6 h-6 animate-spin text-blue-500 mr-2" />
          <span className="text-gray-600">Loading transactions...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow-lg border border-gray-200">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-800 flex items-center">
          <History className="w-6 h-6 mr-2 text-indigo-500" />
          Transaction History
        </h2>
        <button
          onClick={refresh}
          className="flex items-center px-3 py-2 text-indigo-600 hover:bg-indigo-50 rounded-lg transition-colors"
          title="Refresh"
        >
          <RefreshCw className="w-4 h-4" />
        </button>
      </div>

      {transactions.length === 0 ? (
        <div className="text-center py-12">
          <History className="w-16 h-16 text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">No transactions yet</p>
          <p className="text-sm text-gray-400 mt-2">Place your first bet to see it here!</p>
        </div>
      ) : (
        <div className="space-y-3 max-h-[600px] overflow-y-auto pr-2">
          {transactions.map((tx) => (
            <TransactionCard key={tx.bet_id} transaction={tx} />
          ))}
        </div>
      )}

      <div className="mt-4 pt-4 border-t border-gray-200">
        <div className="flex items-center justify-between text-xs text-gray-500">
          <span>Total: {transactions.length} transaction{transactions.length !== 1 ? 's' : ''}</span>
          <span>Auto-refreshes every 5 seconds</span>
        </div>
      </div>
    </div>
  );
}

function TransactionCard({ transaction }: { transaction: Bet }) {
  const stageHint = getStageHint(transaction);
  return (
    <div className="p-4 border-2 border-gray-200 rounded-lg hover:border-indigo-300 transition-colors bg-gradient-to-r from-white to-gray-50">
      <div className="flex justify-between items-start mb-3">
        <div className="flex-1">
          <div className="flex items-center space-x-2 mb-1">
            <span className="font-bold text-gray-800">
              {(transaction.stake_amount / 1_000_000_000).toFixed(3)} SOL
            </span>
            <span className="text-gray-400">•</span>
            <span className="text-sm font-semibold text-gray-600 uppercase">
              {transaction.choice}
            </span>
          </div>
          <div className="text-xs text-gray-500">
            {new Date(transaction.created_at).toLocaleString()}
          </div>
          {stageHint && (
            <div className="mt-1 text-xs text-gray-500">
              {stageHint}
            </div>
          )}
        </div>
        <StatusBadge status={transaction.status} />
      </div>

      <div className="space-y-2">
        {/* Bet ID */}
        <div className="flex items-center justify-between text-xs">
          <span className="text-gray-500">Bet ID:</span>
          <code className="font-mono text-gray-700">{transaction.bet_id.slice(0, 16)}...</code>
        </div>

        {/* Vault */}
        {transaction.vault_address && (
          <div className="flex items-center justify-between text-xs">
            <span className="text-gray-500">Vault:</span>
            <code className="font-mono text-gray-700">{transaction.vault_address.slice(0, 8)}...{transaction.vault_address.slice(-8)}</code>
          </div>
        )}

        {/* Batch / Processor */}
        {(transaction.external_batch_id || transaction.processor_id) && (
          <div className="grid grid-cols-1 gap-1 text-xs">
            {transaction.external_batch_id && (
              <div className="flex items-center justify-between">
                <span className="text-gray-500">Batch:</span>
                <code className="font-mono text-gray-700">{transaction.external_batch_id.slice(0, 8)}...{transaction.external_batch_id.slice(-8)}</code>
              </div>
            )}
            {transaction.processor_id && (
              <div className="flex items-center justify-between">
                <span className="text-gray-500">Processor:</span>
                <span className="text-gray-700">{transaction.processor_id}</span>
              </div>
            )}
          </div>
        )}

        {/* Game Result */}
        {transaction.won !== null && (
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-500">Result:</span>
            <span className={`font-bold ${transaction.won ? 'text-green-600' : 'text-red-600'}`}>
              {transaction.won ? '✓ WON' : '✗ LOST'}
            </span>
          </div>
        )}

        {/* Payout */}
        {transaction.payout_amount !== null && transaction.payout_amount > 0 && (
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-500">Payout:</span>
            <span className="font-semibold text-green-600">
              +{(transaction.payout_amount / 1_000_000_000).toFixed(3)} SOL
            </span>
          </div>
        )}

        {/* Transaction ID */}
        {transaction.solana_tx_id && (
          <div className="mt-2 pt-2 border-t border-gray-200">
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-500">Settlement TX:</span>
              <a
                href={solanaService.getExplorerUrl(transaction.solana_tx_id)}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center text-xs text-blue-600 hover:text-blue-800 font-mono"
              >
                {transaction.solana_tx_id.slice(0, 8)}...{transaction.solana_tx_id.slice(-8)}
                <ExternalLink className="w-3 h-3 ml-1" />
              </a>
            </div>
          </div>
        )}

        {/* Retry Count */}
        {transaction.retry_count > 0 && (
          <div className="flex items-center justify-between text-xs">
            <span className="text-gray-500">Retries:</span>
            <span className="text-orange-600 font-semibold">{transaction.retry_count}</span>
          </div>
        )}

        {/* Error Message */}
        {transaction.last_error_message && (
          <div className="mt-2 pt-2 border-t border-red-200">
            <p className="text-xs text-red-600 bg-red-50 p-2 rounded">
              ⚠️ {transaction.last_error_message}
            </p>
            {transaction.last_error_code && (
              <p className="text-xs text-gray-500 mt-1">
                Code: {transaction.last_error_code}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: BetStatus }) {
  const getStatusConfig = (status: BetStatus) => {
    const normalized = String(status).toLowerCase();
    switch (status) {
      case 'Completed':
        return { text: 'Completed', class: 'bg-green-100 text-green-800 border-green-300' };
      case 'Pending':
        return { text: 'Pending', class: 'bg-yellow-100 text-yellow-800 border-yellow-300' };
      case 'Batched':
        return { text: 'Batched', class: 'bg-blue-100 text-blue-800 border-blue-300' };
      case 'SubmittedToSolana':
        return { text: 'Submitted', class: 'bg-indigo-100 text-indigo-800 border-indigo-300' };
      case 'ConfirmedOnSolana':
        return { text: 'Confirmed', class: 'bg-purple-100 text-purple-800 border-purple-300' };
      case 'FailedRetryable':
        return { text: 'Retrying', class: 'bg-orange-100 text-orange-800 border-orange-300' };
      case 'FailedManualReview':
        return { text: 'Failed', class: 'bg-red-100 text-red-800 border-red-300' };
      default:
        // Backend returns lowercase snake_case statuses
        switch (normalized) {
          case 'completed':
            return { text: 'Completed', class: 'bg-green-100 text-green-800 border-green-300' };
          case 'pending':
            return { text: 'Pending', class: 'bg-yellow-100 text-yellow-800 border-yellow-300' };
          case 'batched':
            return { text: 'Batched', class: 'bg-blue-100 text-blue-800 border-blue-300' };
          case 'submitted_to_solana':
            return { text: 'Submitted', class: 'bg-indigo-100 text-indigo-800 border-indigo-300' };
          case 'confirmed_on_solana':
            return { text: 'Confirmed', class: 'bg-purple-100 text-purple-800 border-purple-300' };
          case 'failed_retryable':
            return { text: 'Retrying', class: 'bg-orange-100 text-orange-800 border-orange-300' };
          case 'failed_manual_review':
            return { text: 'Failed', class: 'bg-red-100 text-red-800 border-red-300' };
          default:
            return { text: String(status), class: 'bg-gray-100 text-gray-800 border-gray-300' };
        }
    }
  };

  const config = getStatusConfig(status);

  return (
    <span className={`px-3 py-1 rounded-full text-xs font-bold border-2 ${config.class}`}>
      {config.text}
    </span>
  );
}

function getStageHint(bet: Bet): string | null {
  const normalized = String(bet.status).toLowerCase();
  switch (normalized) {
    case 'pending':
      return 'Waiting for processor to pick up bet.';
    case 'batched':
      return 'Processor batched this bet for submission.';
    case 'submitted_to_solana':
      return 'Submitted to Solana; awaiting confirmation.';
    case 'confirmed_on_solana':
      return 'Confirmed on Solana; waiting for backend to finalize result.';
    case 'completed':
      return bet.won === null ? 'Completed.' : bet.won ? 'Completed: won.' : 'Completed: lost.';
    case 'failed_retryable':
      return 'Temporary failure; processor will retry.';
    case 'failed_manual_review':
      return 'Failed; needs manual review.';
    default:
      // legacy TitleCase variants
      switch (String(bet.status)) {
        case 'Pending':
          return 'Waiting for processor to pick up bet.';
        case 'Batched':
          return 'Processor batched this bet for submission.';
        case 'SubmittedToSolana':
          return 'Submitted to Solana; awaiting confirmation.';
        case 'ConfirmedOnSolana':
          return 'Confirmed on Solana; waiting for backend to finalize result.';
        case 'Completed':
          return bet.won === null ? 'Completed.' : bet.won ? 'Completed: won.' : 'Completed: lost.';
        case 'FailedRetryable':
          return 'Temporary failure; processor will retry.';
        case 'FailedManualReview':
          return 'Failed; needs manual review.';
        default:
          return null;
      }
  }
}