import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { createAtomikSDK, type AtomikSDK, type AtomikConfig } from "../sdk";
import {
  createTransactionUtils,
  TransactionHelpers,
  type TransactionResult,
} from "../utils/transactions";
import { ReusableWalletConnect } from "./ReusableWalletConnect";
import { Loader2, Send, Coins } from "lucide-react";

/**
 * Complete demo showing how to use all the reusable components together
 * This can be copied to any new Solana project as a starting point
 */
export function ReusableSDKDemo() {
  const { publicKey } = useWallet();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<TransactionResult | null>(null);
  const [transferAmount, setTransferAmount] = useState("0.1");
  const [recipientAddress, setRecipientAddress] = useState("");

  // Configure SDK for your project (manual config for demo)
  const config: AtomikConfig = {
    api: {
      baseUrl: "https://api.example.com",
      timeout: 10000,
      retryAttempts: 3,
    },
    blockchain: {
      network: "devnet",
      rpcUrl: "https://api.devnet.solana.com",
      programId: "11111111111111111111111111111111",
      commitment: "confirmed",
      confirmTimeout: 30000,
    },
    websocket: {
      enabled: true,
      reconnectAttempts: 5,
      reconnectDelay: 1000,
      connectionTimeout: 10000,
    },
  };

  // Create SDK instance
  const sdk: AtomikSDK = createAtomikSDK(config);
  const txUtils = createTransactionUtils(config.blockchain);

  const handleTransfer = async () => {
    if (!publicKey || !recipientAddress || !transferAmount) {
      alert("Please fill in all fields");
      return;
    }

    if (!TransactionHelpers.isValidPublicKey(recipientAddress)) {
      alert("Invalid recipient address");
      return;
    }

    setLoading(true);
    setResult(null);

    try {
      const recipientPubkey = new PublicKey(recipientAddress);
      const lamports = TransactionHelpers.solToLamports(
        parseFloat(transferAmount),
      );

      // Check balance first
      const balance = await txUtils.getBalance(publicKey);
      const fee = 0.000005; // Estimated transaction fee

      if (balance < parseFloat(transferAmount) + fee) {
        throw new Error(
          `Insufficient funds. Need ${parseFloat(transferAmount) + fee} SOL, have ${balance} SOL`,
        );
      }

      // Create and send transaction
      const transaction = await txUtils.createTransferTransaction(
        publicKey,
        recipientPubkey,
        lamports,
        `Transfer ${transferAmount} SOL via Reusable SDK`,
      );

      // Sign and send (in real app, you'd use wallet.signTransaction)
      console.log("Transaction created:", transaction);

      // For demo purposes, just show success
      const mockResult: TransactionResult = {
        success: true,
        signature: "demo_signature_" + Date.now(),
        blockhash: transaction.recentBlockhash || "demo_blockhash",
      };

      setResult(mockResult);
      alert("Demo: Transaction would be successful in real implementation");
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      setResult({
        success: false,
        error: errorMessage,
      });
      console.error("Transfer failed:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleAirdrop = async () => {
    if (!publicKey || config.blockchain.network !== "devnet") return;

    setLoading(true);
    try {
      const signature = await txUtils.requestAirdrop(publicKey, 1);
      setResult({
        success: true,
        signature,
      });
      alert(`Airdrop successful! Signature: ${signature}`);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      setResult({
        success: false,
        error: errorMessage,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-8">
      <div className="text-center">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">
          Reusable Solana SDK Demo
        </h1>
        <p className="text-gray-600">
          Complete example showing wallet connection, transactions, and SDK
          usage
        </p>
      </div>

      {/* Wallet Connection */}
      <div className="bg-white rounded-lg border shadow-sm p-6">
        <h2 className="text-xl font-semibold mb-4">Wallet Connection</h2>
        <ReusableWalletConnect
          config={config}
          showBalance={true}
          showAirdrop={config.blockchain.network === "devnet"}
          showExplorer={true}
          onWalletConnect={(pubkey) => console.log("Connected to:", pubkey)}
          onBalanceUpdate={(balance) =>
            console.log("Balance updated:", balance)
          }
        />
      </div>

      {publicKey && (
        <>
          {/* Transaction Demo */}
          <div className="bg-white rounded-lg border shadow-sm p-6">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
              <Send className="w-5 h-5" />
              Transfer SOL
            </h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Recipient Address
                </label>
                <input
                  type="text"
                  value={recipientAddress}
                  onChange={(e) => setRecipientAddress(e.target.value)}
                  placeholder="Enter Solana address"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Amount (SOL)
                </label>
                <input
                  type="number"
                  value={transferAmount}
                  onChange={(e) => setTransferAmount(e.target.value)}
                  step="0.01"
                  min="0"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500"
                />
              </div>
              <button
                onClick={handleTransfer}
                disabled={loading || !recipientAddress || !transferAmount}
                className="w-full bg-purple-600 text-white py-2 px-4 rounded hover:bg-purple-700 disabled:opacity-50 flex items-center justify-center gap-2"
              >
                {loading && <Loader2 className="w-4 h-4 animate-spin" />}
                Create Transfer Transaction
              </button>
            </div>
          </div>

          {/* SDK Features Demo */}
          <div className="bg-white rounded-lg border shadow-sm p-6">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
              <Coins className="w-5 h-5" />
              SDK Features
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {config.blockchain.network === "devnet" && (
                <button
                  onClick={handleAirdrop}
                  disabled={loading}
                  className="bg-green-600 text-white py-2 px-4 rounded hover:bg-green-700 disabled:opacity-50 flex items-center justify-center gap-2"
                >
                  {loading && <Loader2 className="w-4 h-4 animate-spin" />}
                  Request Airdrop
                </button>
              )}
              <button
                onClick={() => console.log("SDK Config:", config)}
                className="bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700"
              >
                Log SDK Config
              </button>
              <button
                onClick={() => console.log("API Client:", sdk.api)}
                className="bg-indigo-600 text-white py-2 px-4 rounded hover:bg-indigo-700"
              >
                Test API Client
              </button>
              <button
                onClick={() => console.log("Vault Service:", sdk.vault)}
                className="bg-purple-600 text-white py-2 px-4 rounded hover:bg-purple-700"
              >
                Test Vault Service
              </button>
            </div>
          </div>

          {/* Transaction Result */}
          {result && (
            <div className="bg-white rounded-lg border shadow-sm p-6">
              <h2 className="text-xl font-semibold mb-4">Transaction Result</h2>
              {result.success ? (
                <div className="bg-green-50 border border-green-200 rounded p-4">
                  <div className="text-green-800">
                    <p className="font-semibold">✅ Transaction Successful!</p>
                    {result.signature && (
                      <p className="text-sm mt-2">
                        <span className="font-medium">Signature:</span>{" "}
                        <span className="font-mono break-all">
                          {result.signature}
                        </span>
                      </p>
                    )}
                    {result.blockhash && (
                      <p className="text-sm mt-1">
                        <span className="font-medium">Blockhash:</span>{" "}
                        <span className="font-mono break-all">
                          {result.blockhash}
                        </span>
                      </p>
                    )}
                  </div>
                </div>
              ) : (
                <div className="bg-red-50 border border-red-200 rounded p-4">
                  <div className="text-red-800">
                    <p className="font-semibold">❌ Transaction Failed</p>
                    <p className="text-sm mt-2">{result.error}</p>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Usage Examples */}
          <div className="bg-gray-50 rounded-lg border p-6">
            <h2 className="text-xl font-semibold mb-4">Code Examples</h2>
            <div className="space-y-4 text-sm">
              <div>
                <h3 className="font-medium text-gray-700 mb-2">
                  Transaction Utils:
                </h3>
                <pre className="bg-white p-3 rounded border overflow-x-auto">
                  {`const txUtils = createTransactionUtils(config.blockchain);
const tx = await txUtils.createTransferTransaction(from, to, amount, memo);
const result = await txUtils.sendAndConfirmTransaction(tx, signers);`}
                </pre>
              </div>
              <div>
                <h3 className="font-medium text-gray-700 mb-2">
                  SDK Services:
                </h3>
                <pre className="bg-white p-3 rounded border overflow-x-auto">
                  {`const sdk = createAtomikSDK(config);
const vaultInfo = await sdk.vault.getVaultInfo(userPubkey);
const apiResponse = await sdk.api.get('/endpoint');`}
                </pre>
              </div>
              <div>
                <h3 className="font-medium text-gray-700 mb-2">
                  Wallet Component:
                </h3>
                <pre className="bg-white p-3 rounded border overflow-x-auto">
                  {`<ReusableWalletConnect
  config={config}
  onWalletConnect={(pubkey) => console.log(pubkey)}
  onBalanceUpdate={(balance) => updateUI(balance)}
/>`}
                </pre>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export default ReusableSDKDemo;
