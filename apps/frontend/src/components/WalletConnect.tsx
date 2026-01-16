'use client';

import { usePrivy, useWallets } from '@privy-io/react-auth';

export function WalletConnect() {
  const { ready, authenticated, login, logout } = usePrivy();
  const { wallets } = useWallets();

  const connectedWallet = wallets.find((w) => w.walletClientType === 'solana');

  if (!ready) {
    return <div>Loading...</div>;
  }

  if (!authenticated) {
    return (
      <div className="flex flex-col items-center space-y-4">
        <button
          onClick={login}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-semibold transition-colors"
        >
          Connect Wallet
        </button>
        <p className="text-sm text-gray-400">Connect with Phantom or Solflare</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {connectedWallet && (
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="text-sm text-gray-400 mb-1">Connected Wallet</div>
          <div className="font-mono text-sm break-all">
            {connectedWallet.address.slice(0, 8)}...{connectedWallet.address.slice(-8)}
          </div>
        </div>
      )}
      <button
        onClick={logout}
        className="w-full px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg font-semibold transition-colors"
      >
        Disconnect
      </button>
    </div>
  );
}
