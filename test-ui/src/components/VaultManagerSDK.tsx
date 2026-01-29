import { useState } from "react";
import { 
  AtomikConfig, 
  createVaultService
} from "../sdk/index";
import { MemoMessages } from "../sdk/utils/memo";
import { Loader2, ExternalLink } from "lucide-react";

interface VaultManagerSDKProps {
  config: AtomikConfig;
  userPublicKey: string | null;
}

/**
 * Example component demonstrating the new SDK usage
 * This shows how clean and simple the new API is to use
 */
export function VaultManagerSDK({
  config,
  userPublicKey,
}: VaultManagerSDKProps) {
  const [depositAmount, setDepositAmount] = useState("");
  const [withdrawAmount, setWithdrawAmount] = useState("");
  const [betAmount, setBetAmount] = useState("");
  const [betChoice, setBetChoice] = useState<"heads" | "tails">("heads");
  const [vaultPda] = useState<string>("");
  const [balance] = useState<number>(0);
  const [vaultExists] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);

  // Create vault service
  const vaultService = createVaultService(config);

  const handleInitialize = async () => {
    if (!userPublicKey) {
      alert("Please connect wallet first");
      return;
    }

    try {
      setLoading(true);
      // This would implement the full initialization logic
      // When implemented, the user will see this message in their wallet:
      const message = MemoMessages.initializeVault();
      alert(`Transaction will show: "${message}"\n\nFeature placeholder - not yet implemented`);
    } catch (error: any) {
      alert(`Error: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDeposit = async () => {
    if (!userPublicKey || !depositAmount) return;

    const amount = parseFloat(depositAmount);
    if (amount <= 0) return;

    try {
      setLoading(true);
      // This would implement the full deposit logic
      // When implemented, the user will see this message in their wallet:
      const message = MemoMessages.depositSol(amount);
      alert(`Transaction will show: "${message}"\n\nFeature placeholder - not yet implemented`);
      setDepositAmount("");
    } catch (error: any) {
      alert(`Error: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleWithdraw = async () => {
    if (!userPublicKey || !withdrawAmount) return;

    const amount = parseFloat(withdrawAmount);
    if (amount <= 0) return;

    try {
      setLoading(true);
      // This would implement the full withdraw logic
      // When implemented, the user will see this message in their wallet:
      const message = MemoMessages.withdrawSol(amount);
      alert(`Transaction will show: "${message}"\n\nFeature placeholder - not yet implemented`);
      setWithdrawAmount("");
    } catch (error: any) {
      alert(`Error: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleAirdrop = async () => {
    if (!userPublicKey) return;

    try {
      setLoading(true);
      // Note: Airdrop is a system call, not a user transaction,
      // so it won't show a memo in the wallet. The memo system
      // is for user-initiated transactions that need approval.
      const signature = await vaultService.requestAirdrop(userPublicKey);
      alert(`Airdrop requested! Signature: ${signature}\n\nNote: Airdrops don't show memos since they're system calls`);
    } catch (error: any) {
      alert(`Error: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleBet = async () => {
    if (!userPublicKey || !betAmount) return;

    const amount = parseFloat(betAmount);
    if (amount <= 0) return;

    try {
      setLoading(true);
      // This would implement the full betting logic
      // When implemented, the user will see this message in their wallet:
      const message = MemoMessages.placeBet(betChoice, amount);
      alert(`Transaction will show: "${message}"\n\nFeature placeholder - not yet implemented`);
      setBetAmount("");
    } catch (error: any) {
      alert(`Error: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleApproveAllowance = async () => {
    if (!userPublicKey) return;

    try {
      setLoading(true);
      // This would implement the allowance approval logic
      // When implemented, the user will see this message in their wallet:
      const message = MemoMessages.approveAllowance(1.0); // Example: approve 1 SOL
      alert(`Transaction will show: "${message}"\n\nFeature placeholder - not yet implemented`);
    } catch (error: any) {
      alert(`Error: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const getExplorerUrl = (address: string) => {
    const base =
      config.solana.network === "devnet"
        ? "https://explorer.solana.com/address"
        : "https://explorer.solana.com/address";
    return `${base}/${address}${config.solana.network === "devnet" ? "?cluster=devnet" : ""}`;
  };

  if (!userPublicKey) {
    return (
      <div className="p-4 bg-yellow-50 border border-yellow-200 rounded">
        <p className="text-yellow-700">
          Please connect your wallet to use the Vault Manager
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6 bg-white rounded-lg shadow">
      <div className="border-b pb-4">
        <h2 className="text-xl font-semibold">Vault Manager SDK Demo</h2>
        <p className="text-sm text-gray-600 mt-1">
          Clean, simple API for vault operations
        </p>
      </div>

      {/* Vault Info Section */}
      <div className="space-y-4 mb-6">
        <div>
          <label className="block text-sm font-medium mb-1">Vault PDA</label>
          <div className="flex items-center gap-2">
            <code className="flex-1 bg-gray-100 p-2 rounded text-sm font-mono break-all">
              {vaultPda || "Not generated yet"}
            </code>
            {vaultPda && (
              <a
                href={getExplorerUrl(vaultPda)}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 hover:text-blue-800"
              >
                <ExternalLink className="w-4 h-4" />
              </a>
            )}
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium mb-1">
              Vault Status
            </label>
            <span
              className={`inline-flex px-2 py-1 rounded text-xs font-medium ${
                vaultExists
                  ? "bg-green-100 text-green-800"
                  : "bg-red-100 text-red-800"
              }`}
            >
              {loading
                ? "Checking..."
                : vaultExists
                  ? "Initialized"
                  : "Not Created"}
            </span>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">
              SOL Balance
            </label>
            <p className="text-sm font-mono">{balance.toFixed(4)} SOL</p>
          </div>
        </div>
      </div>

      {/* Actions Section */}
      <div className="space-y-4">
        {!vaultExists && (
          <button
            onClick={handleInitialize}
            disabled={loading}
            className="w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {loading && <Loader2 className="w-4 h-4 animate-spin" />}
            Initialize Vault
          </button>
        )}

        <div className="space-y-2">
          <label className="block text-sm font-medium">Deposit SOL</label>
          <div className="flex gap-2">
            <input
              type="number"
              value={depositAmount}
              onChange={(e) => setDepositAmount(e.target.value)}
              placeholder="Amount (SOL)"
              step="0.1"
              min="0"
              className="flex-1 border rounded px-3 py-2"
            />
            <button
              onClick={handleDeposit}
              disabled={
                loading || !depositAmount || parseFloat(depositAmount) <= 0
              }
              className="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700 disabled:opacity-50 flex items-center gap-2"
            >
              {loading && <Loader2 className="w-4 h-4 animate-spin" />}
              Deposit
            </button>
          </div>
        </div>

        <div className="space-y-2">
          <label className="block text-sm font-medium">Withdraw SOL</label>
          <div className="flex gap-2">
            <input
              type="number"
              value={withdrawAmount}
              onChange={(e) => setWithdrawAmount(e.target.value)}
              placeholder="Amount (SOL)"
              step="0.1"
              min="0"
              className="flex-1 border rounded px-3 py-2"
            />
            <button
              onClick={handleWithdraw}
              disabled={
                loading || !withdrawAmount || parseFloat(withdrawAmount) <= 0
              }
              className="bg-orange-600 text-white px-4 py-2 rounded hover:bg-orange-700 disabled:opacity-50 flex items-center gap-2"
            >
              {loading && <Loader2 className="w-4 h-4 animate-spin" />}
              Withdraw
            </button>
          </div>
        </div>

        {/* Betting Section */}
        <div className="border-t pt-4">
          <h3 className="text-lg font-medium mb-3">Betting Demo</h3>
          
          <div className="space-y-2 mb-3">
            <label className="block text-sm font-medium">Approve Session Key (one-time)</label>
            <button
              onClick={handleApproveAllowance}
              disabled={loading}
              className="w-full bg-purple-600 text-white py-2 px-4 rounded hover:bg-purple-700 disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {loading && <Loader2 className="w-4 h-4 animate-spin" />}
              Approve Session Key (1 SOL)
            </button>
            <p className="text-xs text-gray-600">
              Session keys allow faster betting without wallet approval for each bet
            </p>
          </div>

          <div className="space-y-2">
            <label className="block text-sm font-medium">Place Coinflip Bet</label>
            <div className="flex gap-2 mb-2">
              <select 
                value={betChoice} 
                onChange={(e) => setBetChoice(e.target.value as "heads" | "tails")}
                className="border rounded px-3 py-2"
              >
                <option value="heads">Heads</option>
                <option value="tails">Tails</option>
              </select>
              <input
                type="number"
                value={betAmount}
                onChange={(e) => setBetAmount(e.target.value)}
                placeholder="Amount (SOL)"
                step="0.1"
                min="0"
                className="flex-1 border rounded px-3 py-2"
              />
              <button
                onClick={handleBet}
                disabled={loading || !betAmount || parseFloat(betAmount) <= 0}
                className="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
              >
                {loading && <Loader2 className="w-4 h-4 animate-spin" />}
                Bet
              </button>
            </div>
          </div>
        </div>

        {config.solana.network === "devnet" && (
          <button
            onClick={handleAirdrop}
            disabled={loading}
            className="w-full bg-purple-600 text-white py-2 px-4 rounded hover:bg-purple-700 disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {loading && <Loader2 className="w-4 h-4 animate-spin" />}
            Request Airdrop (1 SOL)
          </button>
        )}
      </div>

      <div className="mt-4 p-3 bg-gray-50 rounded">
        <h3 className="text-sm font-medium mb-2">SDK Demo Status</h3>
        <p className="text-xs text-gray-600">
          This is a simplified demo component showing the clean SDK API
          structure. The actual vault operations would use the full
          implementation with transaction building, error handling, and
          confirmation logic from the existing SolanaService.
        </p>
      </div>
    </div>
  );
}
