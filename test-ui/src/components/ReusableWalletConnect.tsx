import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { Wallet, Loader2, ExternalLink } from "lucide-react";
import { useState, useEffect } from "react";
import {
  AtomikConfig,
  AtomikSolanaConfig,
  getBlockchainConfig,
  BlockchainConfig,
} from "../sdk";

export interface WalletConnectConfig {
  blockchain: BlockchainConfig;
  showBalance?: boolean;
  showAirdrop?: boolean;
  showExplorer?: boolean;
  className?: string;
  onBalanceUpdate?: (balance: number) => void;
  onWalletConnect?: (publicKey: string) => void;
  onWalletDisconnect?: () => void;
}

export interface WalletConnectProps {
  config: AtomikConfig | AtomikSolanaConfig | WalletConnectConfig;
  showBalance?: boolean;
  showAirdrop?: boolean;
  showExplorer?: boolean;
  className?: string;
  onBalanceUpdate?: (balance: number) => void;
  onWalletConnect?: (publicKey: string) => void;
  onWalletDisconnect?: () => void;
}

/**
 * Reusable wallet connection component that works with any Solana project
 * Supports both legacy AtomikConfig/AtomikSolanaConfig and generic WalletConnectConfig
 */
export function ReusableWalletConnect({
  config,
  showBalance = true,
  showAirdrop = true,
  showExplorer = true,
  className = "",
  onBalanceUpdate,
  onWalletConnect,
  onWalletDisconnect,
}: WalletConnectProps) {
  const { publicKey, connected } = useWallet();
  const { connection } = useConnection();
  const [balance, setBalance] = useState<number | null>(null);
  const [rpcError, setRpcError] = useState<string>("");
  const [loading, setLoading] = useState(false);

  // Extract blockchain configuration from different config types
  const blockchainConfig =
    "blockchain" in config
      ? config.blockchain
      : "api" in config || "solana" in config
        ? getBlockchainConfig(config as AtomikConfig | AtomikSolanaConfig)
        : (config as BlockchainConfig);

  const isDevnet = blockchainConfig.network === "devnet";

  useEffect(() => {
    if (connected && publicKey) {
      onWalletConnect?.(publicKey.toBase58());
    } else if (!connected) {
      onWalletDisconnect?.();
      setBalance(null);
      setRpcError("");
    }
  }, [connected, publicKey, onWalletConnect, onWalletDisconnect]);

  useEffect(() => {
    if (publicKey && showBalance) {
      setRpcError("");
      setBalance(null);

      connection
        .getBalance(publicKey)
        .then((lamports) => {
          const sol = lamports / 1_000_000_000; // Convert lamports to SOL
          setBalance(sol);
          onBalanceUpdate?.(sol);
        })
        .catch((err) => {
          const msg = err instanceof Error ? err.message : String(err);
          setRpcError(
            msg.includes("429") ||
              msg.includes('code": 429') ||
              msg.toLowerCase().includes("too many requests")
              ? `RPC rate-limited (429) from ${blockchainConfig.rpcUrl}`
              : msg,
          );
        });
    }
  }, [
    publicKey,
    connection,
    showBalance,
    blockchainConfig.rpcUrl,
    onBalanceUpdate,
  ]);

  const handleAirdrop = async () => {
    if (!publicKey || !isDevnet) return;

    setLoading(true);
    try {
      setRpcError("");

      // Request airdrop (1 SOL = 1_000_000_000 lamports)
      const signature = await connection.requestAirdrop(
        publicKey,
        1_000_000_000,
      );

      // Wait for confirmation
      await connection.confirmTransaction(signature);

      console.log("Airdrop successful:", signature);

      // Refresh balance
      const lamports = await connection.getBalance(publicKey);
      const sol = lamports / 1_000_000_000;
      setBalance(sol);
      onBalanceUpdate?.(sol);
    } catch (error) {
      console.error("Airdrop failed:", error);
      const msg = error instanceof Error ? error.message : String(error);
      setRpcError(msg);
    } finally {
      setLoading(false);
    }
  };

  const getExplorerUrl = (address: string) => {
    const base = "https://explorer.solana.com/address";
    return `${base}/${address}${isDevnet ? "?cluster=devnet" : ""}`;
  };

  return (
    <div className={`flex flex-col items-center space-y-4 ${className}`}>
      <div className="flex items-center space-x-4">
        <Wallet className="w-6 h-6 text-purple-600" />
        <WalletMultiButton className="!bg-purple-600 !hover:bg-purple-700" />
      </div>

      {connected && publicKey && (
        <div className="bg-white rounded-lg border shadow-sm p-4 w-full max-w-md">
          <div className="space-y-3">
            <div>
              <div className="text-sm text-gray-500">Wallet Address</div>
              <div className="font-mono text-sm break-all bg-gray-50 p-2 rounded">
                {publicKey.toBase58()}
              </div>
              {showExplorer && (
                <a
                  href={getExplorerUrl(publicKey.toBase58())}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-purple-600 hover:text-purple-700 text-sm mt-1"
                >
                  View on Explorer <ExternalLink className="w-3 h-3" />
                </a>
              )}
            </div>

            {showBalance && (
              <div>
                <div className="text-sm text-gray-500">Balance</div>
                <div className="text-lg font-semibold">
                  {balance !== null
                    ? `${balance.toFixed(4)} SOL`
                    : "Loading..."}
                </div>
                {rpcError && (
                  <div className="text-red-600 text-sm mt-1 p-2 bg-red-50 rounded">
                    {rpcError}
                  </div>
                )}
              </div>
            )}
          </div>

          {showAirdrop && isDevnet && (
            <button
              onClick={handleAirdrop}
              disabled={loading || !publicKey}
              className="w-full mt-4 bg-purple-600 text-white py-2 px-4 rounded hover:bg-purple-700 disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {loading && <Loader2 className="w-4 h-4 animate-spin" />}
              Request Airdrop (1 SOL)
            </button>
          )}

          {isDevnet && (
            <div className="mt-3 text-sm text-amber-600 bg-amber-50 p-2 rounded border border-amber-200">
              You are on Solana Devnet. Transactions use test SOL.
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/**
 * Legacy wrapper for backward compatibility with existing casino code
 */
export function WalletConnect() {
  // Use environment-based config for legacy compatibility
  const config: WalletConnectConfig = {
    blockchain: {
      network: "devnet", // Default to devnet for casino
      rpcUrl:
        import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com",
      programId: import.meta.env.VITE_VAULT_PROGRAM_ID || "",
      commitment: "confirmed",
      confirmTimeout: 30000,
    },
  };

  return <ReusableWalletConnect config={config} />;
}

export default ReusableWalletConnect;
