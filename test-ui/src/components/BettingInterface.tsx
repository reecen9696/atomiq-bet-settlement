import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { useApi } from "../hooks/useApi";
import {
  Coins,
  Loader2,
  TrendingUp,
  TrendingDown,
  AlertCircle,
} from "lucide-react";
import type { GameResponse } from "../types";

interface BettingInterfaceProps {
  allowanceExists?: boolean | null;
  allowanceRemaining?: bigint | null;
}

export function BettingInterface({
  allowanceExists,
  // allowanceRemaining, // TODO: Use for display
}: BettingInterfaceProps) {
  const { publicKey } = useWallet();
  const { playCoinflip, getGameResult, isLoading } = useApi();
  const [amount, setAmount] = useState("0.01");
  const [choice, setChoice] = useState<"heads" | "tails">("heads");
  const [lastResponse, setLastResponse] = useState<GameResponse | null>(null);
  const [error, setError] = useState<string>("");

  const parsedAmount = Number.parseFloat(amount);
  const displayAmount = Number.isFinite(parsedAmount) ? parsedAmount : 0;

  const handlePlaceBet = async () => {
    if (!publicKey) {
      setError("Please connect your wallet first");
      return;
    }

    const amountNum = parseFloat(amount);
    if (isNaN(amountNum) || amountNum < 0.01) {
      setError("Minimum bet is 0.01 SOL");
      return;
    }

    setError("");
    try {
      // Retrieve the last approved allowance PDA from localStorage
      const allowancePda =
        localStorage.getItem("lastAllowancePda") || undefined;

      const response = await playCoinflip({
        player_id: publicKey.toBase58(),
        choice,
        token: { symbol: "SOL", mint_address: null },
        bet_amount: amountNum,
        allowance_pda: allowancePda,
      });

      setLastResponse(response);

      // If the game is pending, opportunistically poll once for a completion update.
      if (response.status === "pending") {
        try {
          const next = await getGameResult(response.game_id);
          setLastResponse(next);
        } catch {
          // ignore; the transaction log will show settlement records
        }
      }
    } catch (err) {
      const errorMsg =
        err instanceof Error ? err.message : "Failed to place bet";
      setError(errorMsg);
      console.error("Bet placement error:", err);
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

  const hasAllowance = localStorage.getItem("lastAllowancePda");

  return (
    <div className="p-6 bg-white rounded-lg shadow-lg border border-gray-200">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-800 flex items-center">
          <Coins className="w-6 h-6 mr-2 text-orange-500" />
          Place Bet
        </h2>
      </div>

      {!hasAllowance && (
        <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg flex items-start">
          <AlertCircle className="w-5 h-5 text-yellow-600 mr-2 mt-0.5 flex-shrink-0" />
          <div className="text-sm text-yellow-800">
            <strong>No allowance found.</strong> Please approve an allowance in
            the Vault Manager before placing bets.
          </div>
        </div>
      )}

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
            min="0.01"
            step="0.01"
            className="w-full px-4 py-3 border-2 border-gray-300 rounded-lg focus:border-orange-500 focus:ring-2 focus:ring-orange-200 transition-all text-lg font-mono"
            placeholder="0.01"
          />
          <p className="text-xs text-gray-500 mt-1">
            Minimum: 0.01 SOL (~
            {(parseFloat(amount) * 1_000_000_000).toLocaleString()} lamports)
          </p>
        </div>

        {/* Choice Selection */}
        <div>
          <label className="block text-sm font-semibold text-gray-700 mb-2">
            Your Prediction
          </label>
          <div className="grid grid-cols-2 gap-3">
            <button
              onClick={() => setChoice("heads")}
              className={`p-4 rounded-lg border-2 transition-all ${
                choice === "heads"
                  ? "border-orange-500 bg-orange-50 shadow-md"
                  : "border-gray-300 bg-white hover:border-orange-300"
              }`}
            >
              <div className="text-center">
                <div className="text-3xl mb-2">🪙</div>
                <div className="font-bold text-gray-800">Heads</div>
              </div>
            </button>
            <button
              onClick={() => setChoice("tails")}
              className={`p-4 rounded-lg border-2 transition-all ${
                choice === "tails"
                  ? "border-orange-500 bg-orange-50 shadow-md"
                  : "border-gray-300 bg-white hover:border-orange-300"
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

        {/* Allowance Warning */}
        {allowanceExists === false && (
          <div className="p-3 bg-yellow-50 border border-yellow-300 rounded-lg flex items-start">
            <AlertCircle className="w-5 h-5 text-yellow-600 mr-2 flex-shrink-0 mt-0.5" />
            <div className="text-sm text-yellow-800">
              <strong>No active allowance.</strong> This UI can still create a
              blockchain bet, but Solana settlement may fail until an allowance
              exists.
            </div>
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

        {/* Last Response */}
        {lastResponse && (
          <div
            className={`p-4 rounded-lg border-2 ${
              lastResponse.status === "complete"
                ? "bg-green-50 border-green-300"
                : "bg-blue-50 border-blue-300"
            }`}
          >
            <div className="flex items-start justify-between mb-2">
              <h3 className="font-bold text-gray-800 flex items-center">
                {lastResponse.status === "complete" ? (
                  <TrendingUp className="w-5 h-5 mr-1 text-green-600" />
                ) : (
                  <TrendingDown className="w-5 h-5 mr-1 text-blue-600" />
                )}
                Last Response
              </h3>
              <span
                className={`px-2 py-1 rounded text-xs font-bold ${getStatusBadgeClass(lastResponse.status)}`}
              >
                {lastResponse.status}
              </span>
            </div>

            <div className="space-y-2 text-sm text-black">
              <div className="flex justify-between">
                <span className="text-black">Game ID:</span>
                <code className="text-xs font-mono">
                  {lastResponse.game_id}
                </code>
              </div>
              <div className="flex justify-between">
                <span className="text-black">Amount:</span>
                <span className="font-semibold">
                  {displayAmount.toFixed(3)} SOL
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-black">Choice:</span>
                <span className="font-semibold uppercase">{choice}</span>
              </div>
              {lastResponse.status === "complete" && (
                <>
                  <div className="flex justify-between">
                    <span className="text-black">Outcome:</span>
                    <span className="font-semibold uppercase">
                      {lastResponse.result.outcome}
                    </span>
                  </div>
                  <div className="pt-2 border-t border-gray-300">
                    <p className="text-xs text-black mb-1">VRF Output:</p>
                    <code className="text-xs font-mono break-all text-black">
                      {lastResponse.result.vrf.vrf_output}
                    </code>
                  </div>
                </>
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
    case "complete":
      return "bg-green-100 text-green-800";
    case "pending":
      return "bg-blue-100 text-blue-800";
    default:
      return "bg-gray-100 text-gray-800";
  }
}
