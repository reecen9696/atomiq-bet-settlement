import { useState } from "react";
import { useTransactions } from "../hooks/useTransactions";
import { useRecentGames } from "../hooks/useRecentGames";
import {
  History,
  Loader2,
  RefreshCw,
  ExternalLink,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { solanaService } from "../services/solana";
import { useApi } from "../hooks/useApi";
import type {
  RecentGameSummary,
  SettlementGame,
  SettlementGameDetail,
} from "../types";

export function TransactionLog() {
  const { transactions, isLoading, refresh, currentWallet, error } =
    useTransactions();
  const {
    games: recentGames,
    isLoading: isRecentGamesLoading,
    error: recentGamesError,
    refresh: refreshRecentGames,
  } = useRecentGames();

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
          <p className="text-gray-500">No pending settlements</p>
          <p className="text-sm text-gray-400 mt-2">
            This view shows{" "}
            <span className="font-semibold">/api/settlement/pending</span>. If a
            settlement is already completed, it may not appear here.
          </p>
          {error && (
            <p className="text-sm text-red-600 mt-3">
              Failed to load pending settlements: {error}
            </p>
          )}
        </div>
      ) : (
        <div className="space-y-3 max-h-[600px] overflow-y-auto pr-2">
          {transactions.map((tx) => (
            <TransactionCard
              key={tx.transaction_id}
              transaction={tx}
              currentWallet={currentWallet}
            />
          ))}
        </div>
      )}

      {transactions.length > 0 && error && (
        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-700">
          Failed to refresh pending settlements: {error}
        </div>
      )}

      <div className="mt-4 pt-4 border-t border-gray-200">
        <div className="flex items-center justify-between text-xs text-gray-500">
          <span>
            Total: {transactions.length} transaction
            {transactions.length !== 1 ? "s" : ""}
          </span>
          <span>Auto-refreshes every 5 seconds</span>
        </div>
        {currentWallet && (
          <div className="mt-2 text-[11px] text-gray-500">
            Highlighting records for:{" "}
            <span className="font-mono">{currentWallet}</span>
          </div>
        )}
      </div>

      <div className="mt-8 pt-6 border-t border-gray-200">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-bold text-gray-800">
            Blockchain Bets (Recent)
          </h3>
          <button
            onClick={refreshRecentGames}
            className="flex items-center px-3 py-2 text-indigo-600 hover:bg-indigo-50 rounded-lg transition-colors"
            title="Refresh"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>

        {isRecentGamesLoading && recentGames.length === 0 ? (
          <div className="flex items-center text-sm text-gray-600">
            <Loader2 className="w-4 h-4 animate-spin text-blue-500 mr-2" />
            Loading recent games...
          </div>
        ) : recentGames.length === 0 ? (
          <div className="text-sm text-gray-500">
            No recent games found.
            {recentGamesError && (
              <div className="mt-2 text-sm text-red-600">
                Failed to load recent games: {recentGamesError}
              </div>
            )}
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-gray-500 border-b border-gray-200">
                  <th className="py-2 pr-4">Tx ID</th>
                  <th className="py-2 pr-4">Processed</th>
                  <th className="py-2 pr-4">Settlement</th>
                  <th className="py-2 pr-4">Bet</th>
                  <th className="py-2 pr-4">Player</th>
                  <th className="py-2 pr-4">Solana TX</th>
                </tr>
              </thead>
              <tbody>
                {recentGames.map((g) => (
                  <RecentGameRow
                    key={g.tx_id}
                    game={g}
                    currentWallet={currentWallet}
                  />
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function RecentGameRow({
  game,
  currentWallet,
}: {
  game: RecentGameSummary;
  currentWallet: string | null;
}) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [details, setDetails] = useState<SettlementGameDetail | null>(null);
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);
  const [detailsError, setDetailsError] = useState<string | null>(null);
  const { getSettlementGame } = useApi();

  const isMine = !!currentWallet && game.player_id === currentWallet;
  const processedText =
    typeof game.processed === "boolean"
      ? game.processed
        ? "true"
        : "false"
      : "—";
  const processedClass =
    typeof game.processed === "boolean"
      ? game.processed
        ? "text-green-700"
        : "text-blue-700"
      : "text-gray-500";

  const handleToggle = async () => {
    if (!isExpanded) {
      // Expanding - fetch details if we don't have them yet
      if (!details && !isLoadingDetails) {
        setIsLoadingDetails(true);
        setDetailsError(null);
        try {
          const fetchedDetails = await getSettlementGame(game.tx_id);
          setDetails(fetchedDetails);
        } catch (error) {
          const msg =
            error instanceof Error ? error.message : "Failed to fetch details";
          setDetailsError(msg);
        } finally {
          setIsLoadingDetails(false);
        }
      }
    }
    setIsExpanded(!isExpanded);
  };

  return (
    <>
      <tr
        className={`border-b border-gray-100 ${isMine ? "bg-indigo-50/40" : ""}`}
      >
        <td className="py-2 pr-4">
          <button
            onClick={handleToggle}
            className="inline-flex items-center text-gray-600 hover:text-indigo-600 transition-colors"
            title={isExpanded ? "Collapse" : "Expand details"}
          >
            {isExpanded ? (
              <ChevronDown className="w-4 h-4 mr-1" />
            ) : (
              <ChevronRight className="w-4 h-4 mr-1" />
            )}
            <span className="font-mono text-gray-800">{game.tx_id}</span>
          </button>
        </td>
        <td className={`py-2 pr-4 font-mono ${processedClass}`}>
          {processedText}
        </td>
        <td className="py-2 pr-4">
          <div>
            <StatusBadge status={game.settlement_status || "Unknown"} />
          </div>
          {game.settlement_error && (
            <div className="mt-1 text-xs text-red-700 max-w-xs break-words">
              {game.settlement_error}
            </div>
          )}
        </td>
        <td className="py-2 pr-4 text-gray-700">
          {(game.bet_amount / 1_000_000_000).toFixed(3)}{" "}
          {game.token?.symbol ?? "SOL"}
        </td>
        <td className="py-2 pr-4 font-mono text-gray-700">{game.player_id}</td>
        <td className="py-2 pr-4">
          {game.solana_tx_id ? (
            <a
              href={solanaService.getExplorerUrl(game.solana_tx_id)}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center text-blue-600 hover:text-blue-800 font-mono"
            >
              {game.solana_tx_id.slice(0, 8)}...{game.solana_tx_id.slice(-8)}
              <ExternalLink className="w-3 h-3 ml-1" />
            </a>
          ) : (
            <span className="text-gray-400">—</span>
          )}
        </td>
      </tr>
      {isExpanded && (
        <tr
          className={`border-b border-gray-100 ${isMine ? "bg-indigo-50/20" : "bg-gray-50"}`}
        >
          <td colSpan={6} className="py-4 px-4">
            {isLoadingDetails ? (
              <div className="flex items-center text-sm text-gray-600">
                <Loader2 className="w-4 h-4 animate-spin text-blue-500 mr-2" />
                Loading full details...
              </div>
            ) : detailsError ? (
              <div className="text-sm text-red-600">
                Failed to load details: {detailsError}
              </div>
            ) : details ? (
              <div className="space-y-3 text-sm">
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <span className="text-gray-500 font-medium">
                      Game Type:
                    </span>
                    <span className="ml-2 text-gray-800">
                      {details.game_type}
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500 font-medium">Outcome:</span>
                    <span className="ml-2 text-gray-800 uppercase">
                      {details.outcome}
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500 font-medium">Payout:</span>
                    <span className="ml-2 text-gray-800 font-mono">
                      {(details.payout / 1_000_000_000).toFixed(9)} SOL
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500 font-medium">
                      Block Height:
                    </span>
                    <span className="ml-2 text-gray-800 font-mono">
                      {details.block_height}
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500 font-medium">
                      Block Hash:
                    </span>
                    <span className="ml-2 text-gray-700 font-mono text-xs break-all">
                      {details.block_hash}
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500 font-medium">Version:</span>
                    <span className="ml-2 text-gray-800 font-mono">
                      {details.version}
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500 font-medium">
                      Settlement Status:
                    </span>
                    <span className="ml-2">
                      <StatusBadge
                        status={details.settlement_status || "Unknown"}
                      />
                    </span>
                  </div>
                </div>

                <div className="col-span-2 mt-2 p-2 bg-blue-50 border border-blue-200 rounded text-xs text-blue-800">
                  <strong>Note:</strong> Retry count tracking is handled by the
                  transaction processor service (not blockchain). Settlement
                  status and error information are shown above. Failed bets will
                  retry up to 5 times with exponential backoff.
                </div>

                {details.settlement_completed_at && (
                  <div>
                    <span className="text-gray-500 font-medium">
                      Settlement Completed:
                    </span>
                    <span className="ml-2 text-gray-800">
                      {new Date(
                        details.settlement_completed_at * 1000,
                      ).toLocaleString()}
                    </span>
                  </div>
                )}

                <div className="mt-4 pt-4 border-t border-gray-300">
                  <div className="font-bold text-gray-700 mb-2">
                    VRF Proof (Provable Fairness)
                  </div>
                  <div className="space-y-2">
                    <div>
                      <div className="text-xs text-gray-500 mb-1">
                        VRF Output
                      </div>
                      <code className="block text-[11px] font-mono break-all text-gray-700 bg-gray-100 p-2 rounded">
                        {details.vrf_output}
                      </code>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">
                        VRF Proof
                      </div>
                      <code className="block text-[11px] font-mono break-all text-gray-700 bg-gray-100 p-2 rounded">
                        {details.vrf_proof}
                      </code>
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-sm text-gray-500">No details available</div>
            )}
          </td>
        </tr>
      )}
    </>
  );
}

function TransactionCard({
  transaction,
  currentWallet,
}: {
  transaction: SettlementGame | SettlementGameDetail;
  currentWallet: string | null;
}) {
  const isMine =
    !!currentWallet && transaction.player_address === currentWallet;
  const isPending =
    "_isPending" in (transaction as any)
      ? Boolean((transaction as any)._isPending)
      : true;
  const settlementStatus =
    "settlement_status" in transaction
      ? String(transaction.settlement_status)
      : isPending
        ? "PendingSettlement"
        : "Processing";
  return (
    <div
      className={`p-4 border-2 rounded-lg hover:border-indigo-300 transition-colors bg-gradient-to-r from-white to-gray-50 ${
        isMine ? "border-indigo-300" : "border-gray-200"
      }`}
    >
      <div className="flex justify-between items-start mb-3">
        <div className="flex-1">
          <div className="flex items-center space-x-2 mb-1">
            <span className="font-bold text-gray-800">
              {(transaction.bet_amount / 1_000_000_000).toFixed(3)} SOL
            </span>
            <span className="text-gray-400">•</span>
            <span className="text-sm font-semibold text-gray-600 uppercase">
              {transaction.game_type}
            </span>
          </div>
          <div className="text-xs text-gray-500">
            TX ID:{" "}
            <span className="font-mono">{transaction.transaction_id}</span>
          </div>
          <div className="mt-1 text-xs text-gray-500">
            Player:{" "}
            <span className="font-mono">{transaction.player_address}</span>
          </div>
        </div>
        <StatusBadge status={settlementStatus} />
      </div>

      <div className="space-y-2">
        <div className="grid grid-cols-1 gap-1 text-xs">
          <div className="flex items-center justify-between">
            <span className="text-gray-500">Outcome:</span>
            <span className="text-gray-700 font-semibold uppercase">
              {transaction.outcome}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-500">Payout:</span>
            <span className="text-gray-700 font-mono">
              {transaction.payout}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-500">Block Height:</span>
            <span className="text-gray-700 font-mono">
              {transaction.block_height}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-500">Version:</span>
            <span className="text-gray-700 font-mono">
              {transaction.version}
            </span>
          </div>
        </div>

        {"solana_tx_id" in transaction && transaction.solana_tx_id && (
          <div className="mt-2 pt-2 border-t border-gray-200">
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-500">Solana TX:</span>
              <a
                href={solanaService.getExplorerUrl(transaction.solana_tx_id)}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center text-xs text-blue-600 hover:text-blue-800 font-mono"
              >
                {transaction.solana_tx_id.slice(0, 8)}...
                {transaction.solana_tx_id.slice(-8)}
                <ExternalLink className="w-3 h-3 ml-1" />
              </a>
            </div>
          </div>
        )}

        {"settlement_error" in transaction && transaction.settlement_error && (
          <div className="mt-2 pt-2 border-t border-red-200">
            <p className="text-xs text-red-700 bg-red-50 p-2 rounded">
              Settlement error: {transaction.settlement_error}
            </p>
          </div>
        )}

        <div className="mt-2 pt-2 border-t border-gray-200 space-y-2">
          <div>
            <div className="text-xs text-gray-500 mb-1">VRF Output</div>
            <code className="text-[11px] font-mono break-all text-gray-700">
              {transaction.vrf_output}
            </code>
          </div>
          <div>
            <div className="text-xs text-gray-500 mb-1">VRF Proof</div>
            <code className="text-[11px] font-mono break-all text-gray-700">
              {transaction.vrf_proof}
            </code>
          </div>
        </div>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const normalized = status.toLowerCase();
  const config = (() => {
    switch (normalized) {
      case "pendingsettlement":
        return {
          text: "Pending Settlement",
          class: "bg-blue-100 text-blue-800 border-blue-300",
        };
      case "processing":
        return {
          text: "Processing",
          class: "bg-yellow-100 text-yellow-800 border-yellow-300",
        };
      case "submittedtosolana":
        return {
          text: "Submitted",
          class: "bg-indigo-100 text-indigo-800 border-indigo-300",
        };
      case "settlementcomplete":
        return {
          text: "Complete",
          class: "bg-green-100 text-green-800 border-green-300",
        };
      case "settlementfailed":
        return {
          text: "Failed",
          class: "bg-red-100 text-red-800 border-red-300",
        };
      default:
        return {
          text: status,
          class: "bg-gray-100 text-gray-800 border-gray-300",
        };
    }
  })();

  return (
    <span
      className={`px-3 py-1 rounded-full text-xs font-bold border-2 ${config.class}`}
    >
      {config.text}
    </span>
  );
}
