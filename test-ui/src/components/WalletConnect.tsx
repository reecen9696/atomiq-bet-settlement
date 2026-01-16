import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { Wallet, Loader2 } from 'lucide-react';
import { useState, useEffect } from 'react';
import { solanaService } from '../services/solana';

export function WalletConnect() {
  const { publicKey, connected } = useWallet();
  const { connection } = useConnection();
  const [balance, setBalance] = useState<number | null>(null);
  const [rpcError, setRpcError] = useState<string>('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (publicKey) {
      setRpcError('');
      setBalance(null);
      solanaService
        .getBalance(publicKey.toBase58(), connection)
        .then((b) => setBalance(b))
        .catch((err) => {
          const msg = err instanceof Error ? err.message : String(err);
          setRpcError(
            msg.includes(' 429') || msg.includes('code": 429') || msg.toLowerCase().includes('too many requests')
              ? `RPC rate-limited (429) from ${solanaService.getRpcUrl()}`
              : msg
          );
        });
    }
  }, [publicKey, connection]);

  const handleAirdrop = async () => {
    if (!publicKey) return;
    
    setLoading(true);
    try {
      setRpcError('');
      const signature = await solanaService.requestAirdrop(publicKey.toBase58(), 1, connection);
      console.log('Airdrop successful:', signature);
      
      // Refresh balance
      const newBalance = await solanaService.getBalance(publicKey.toBase58(), connection);
      setBalance(newBalance);
      
      alert(`Airdrop successful! TX: ${signature}`);
    } catch (error) {
      console.error('Airdrop failed:', error);
      const msg = error instanceof Error ? error.message : String(error);
      setRpcError(msg);
      alert('Airdrop failed. Please try again or use https://faucet.solana.com');
    } finally {
      setLoading(false);
    }
  };

  if (!connected) {
    return (
      <div className="p-6 bg-white rounded-lg shadow-lg border border-gray-200">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-800 flex items-center">
            <Wallet className="w-6 h-6 mr-2" />
            Wallet Connection
          </h2>
        </div>
        <p className="text-gray-600 mb-4">
          Connect your wallet to start placing bets
        </p>
        <WalletMultiButton className="!w-full !bg-gradient-to-r !from-blue-500 !to-purple-600 !text-white !px-6 !py-3 !rounded-lg hover:!from-blue-600 hover:!to-purple-700 !transition-all !duration-200 !font-semibold !shadow-md hover:!shadow-lg" />
      </div>
    );
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow-lg border border-gray-200">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold text-gray-800 flex items-center">
          <Wallet className="w-6 h-6 mr-2 text-green-500" />
          Connected
        </h2>
        <WalletMultiButton className="!text-sm" />
      </div>

      <div className="space-y-3">
        {publicKey && (
          <>
            <div className="p-3 bg-gray-50 rounded-lg">
              <p className="text-xs text-gray-500 mb-1">Wallet Address</p>
              <p className="text-sm font-mono text-gray-800 break-all">
                {publicKey.toBase58()}
              </p>
              <a
                href={solanaService.getAccountExplorerUrl(publicKey.toBase58())}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-blue-600 hover:underline mt-1 inline-block"
              >
                View on Explorer →
              </a>
            </div>

            <div className="p-3 bg-gradient-to-r from-green-50 to-blue-50 rounded-lg border border-green-200">
              <div className="flex justify-between items-center">
                <div>
                  <p className="text-xs text-gray-500 mb-1">Balance</p>
                  <p className="text-2xl font-bold text-gray-800">
                    {balance === null ? '—' : `${balance.toFixed(4)} SOL`}
                  </p>
                </div>
                <button
                  onClick={handleAirdrop}
                  disabled={loading}
                  className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors text-sm font-semibold"
                >
                  {loading ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    'Request Airdrop'
                  )}
                </button>
              </div>
              <p className="text-xs text-gray-500 mt-2">
                Request 1 SOL from devnet faucet
              </p>
              {rpcError && (
                <p className="text-xs text-red-700 mt-2 break-words">
                  {rpcError}
                </p>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}