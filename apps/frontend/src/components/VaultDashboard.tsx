'use client';

import { useState } from 'react';
import { useWallets } from '@privy-io/react-auth';

export function VaultDashboard() {
  const { wallets } = useWallets();
  const connectedWallet = wallets.find((w) => w.walletClientType === 'solana');

  // Mock data - replace with actual vault queries
  const [balance, setBalance] = useState({
    sol: 0,
    usdc: 0,
  });

  const [allowance, setAllowance] = useState({
    amount: 0,
    remaining: 0,
    expiresAt: null as Date | null,
  });

  if (!connectedWallet) {
    return null;
  }

  return (
    <div className="bg-gray-900 rounded-lg p-6">
      <h2 className="text-2xl font-bold mb-6">Vault Dashboard</h2>

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="text-sm text-gray-400 mb-1">SOL Balance</div>
          <div className="text-2xl font-bold">{balance.sol.toFixed(4)} SOL</div>
        </div>
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="text-sm text-gray-400 mb-1">USDC Balance</div>
          <div className="text-2xl font-bold">{balance.usdc.toFixed(2)} USDC</div>
        </div>
      </div>

      <div className="space-y-4">
        <div className="bg-gray-800 rounded-lg p-4">
          <h3 className="font-semibold mb-3">Active Allowance</h3>
          {allowance.expiresAt ? (
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-gray-400">Total:</span>
                <span>{allowance.amount} SOL</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-400">Remaining:</span>
                <span className="text-green-400">{allowance.remaining} SOL</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-400">Expires:</span>
                <span>{allowance.expiresAt.toLocaleString()}</span>
              </div>
            </div>
          ) : (
            <div className="text-sm text-gray-400">No active allowance</div>
          )}
        </div>

        <div className="grid grid-cols-3 gap-3">
          <button className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg font-semibold transition-colors">
            Deposit
          </button>
          <button className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg font-semibold transition-colors">
            Approve
          </button>
          <button className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg font-semibold transition-colors">
            Withdraw
          </button>
        </div>
      </div>
    </div>
  );
}
