import { useMemo, useState, useEffect } from "react";
import {
  ConnectionProvider,
  WalletProvider,
  useWallet,
} from "@solana/wallet-adapter-react";
import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
  TorusWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import { clusterApiUrl } from "@solana/web3.js";
import { WalletConnect } from "./components/WalletConnect";
import { VaultManager } from "./components/VaultManager";
import { BettingInterface } from "./components/BettingInterface";
import { TransactionLog } from "./components/TransactionLog";
import { solanaService } from "./services/solana";
import { useConnection } from "@solana/wallet-adapter-react";

import "@solana/wallet-adapter-react-ui/styles.css";

function AppContent() {
  const { publicKey } = useWallet();
  const { connection } = useConnection();
  const [allowanceExists, setAllowanceExists] = useState<boolean | null>(null);
  const [allowanceRemaining, setAllowanceRemaining] = useState<bigint | null>(
    null,
  );
  const envNetwork = (import.meta.env.VITE_SOLANA_NETWORK ||
    "devnet") as string;

  // Check allowance state when wallet changes
  useEffect(() => {
    if (!publicKey) {
      setAllowanceExists(null);
      setAllowanceRemaining(null);
      return;
    }

    const checkAllowance = async () => {
      try {
        const key = `atomik:lastAllowancePda:${publicKey.toBase58()}`;
        const savedPda = localStorage.getItem(key);

        if (!savedPda || savedPda.length < 20) {
          setAllowanceExists(false);
          setAllowanceRemaining(null);
          return;
        }

        const info = await solanaService.getAllowanceInfoByAddress(
          savedPda,
          connection,
        );
        setAllowanceExists(info.exists && !info.state?.revoked);
        setAllowanceRemaining(info.state?.remainingLamports || null);
      } catch (err) {
        console.error("Error checking allowance:", err);
        setAllowanceExists(null);
        setAllowanceRemaining(null);
      }
    };

    checkAllowance();

    // Poll for updates every 5 seconds
    const interval = setInterval(checkAllowance, 5000);
    return () => clearInterval(interval);
  }, [publicKey, connection]);

  return (
    <>
      {/* Header */}
      <header className="bg-white shadow-md border-b-4 border-orange-500">
        <div className="max-w-7xl mx-auto px-4 py-6">
          <h1 className="text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-orange-500 to-purple-600">
            🎲 Atomik Bet Test UI
          </h1>
          <p className="text-gray-600 mt-2">
            Devnet Testing Interface for Solana Betting System
          </p>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          {/* Left Column */}
          <div className="space-y-6">
            <WalletConnect />
            <VaultManager />
            <BettingInterface
              allowanceExists={allowanceExists}
              allowanceRemaining={allowanceRemaining}
            />
          </div>

          {/* Right Column */}
          <div>
            <TransactionLog />
          </div>
        </div>

        {/* Footer Info */}
        <div className="mt-8 p-6 bg-white rounded-lg shadow-lg border-l-4 border-blue-500">
          <h3 className="font-bold text-gray-800 mb-3 flex items-center">
            ℹ️ How to Use This Test UI
          </h3>
          <ol className="list-decimal list-inside space-y-2 text-sm text-gray-700">
            <li>
              <strong>Connect Wallet:</strong> Click "Connect with Privy" to
              create or connect a Solana wallet
            </li>
            <li>
              <strong>Request Airdrop:</strong> Get devnet SOL (1 SOL = test
              funds for devnet)
            </li>
            <li>
              <strong>Check Vault:</strong> Your vault address is automatically
              derived from your wallet
            </li>
            <li>
              <strong>Place Bet:</strong> Choose amount and heads/tails, then
              place your bet
            </li>
            <li>
              <strong>Monitor Results:</strong> Watch the transaction log for
              real-time status updates
            </li>
            <li>
              <strong>Verify On-Chain:</strong> Click transaction links to view
              on Solana Explorer
            </li>
          </ol>
          <div className="mt-4 pt-4 border-t border-gray-200">
            <p className="text-xs text-gray-500">
              <strong>Blockchain API:</strong>{" "}
              {import.meta.env.VITE_API_BASE_URL || "http://localhost:3001"}
              <br />
              <strong>Program ID:</strong>{" "}
              {import.meta.env.VITE_VAULT_PROGRAM_ID}
              <br />
              <strong>Network:</strong> {envNetwork}
            </p>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="mt-12 py-6 text-center text-gray-500 text-sm">
        <p>Atomik Wallet Test Interface • Powered by Solana</p>
      </footer>
    </>
  );
}

function App() {
  const envNetwork = (import.meta.env.VITE_SOLANA_NETWORK ||
    "devnet") as string;
  const network =
    envNetwork === "mainnet-beta"
      ? WalletAdapterNetwork.Mainnet
      : envNetwork === "testnet"
        ? WalletAdapterNetwork.Testnet
        : WalletAdapterNetwork.Devnet;
  const endpoint = useMemo(
    () =>
      (import.meta.env.VITE_SOLANA_RPC_URL as string) || clusterApiUrl(network),
    [network],
  );

  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter({ network }),
      new TorusWalletAdapter(),
    ],
    [network],
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <AppContent />
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}

export default App;
