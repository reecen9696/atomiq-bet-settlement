'use client';

import { WalletConnect } from '@/components/WalletConnect';
import { VaultDashboard } from '@/components/VaultDashboard';
import { BetInterface } from '@/components/BetInterface';
import { BetHistory } from '@/components/BetHistory';
import { usePrivy } from '@privy-io/react-auth';

export default function Home() {
  const { ready, authenticated } = usePrivy();

  if (!ready) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-xl">Loading...</div>
      </div>
    );
  }

  return (
    <main className="min-h-screen p-8">
      <div className="max-w-7xl mx-auto">
        <header className="mb-8">
          <h1 className="text-4xl font-bold mb-2">Atomik Wallet</h1>
          <p className="text-gray-400">Non-custodial Solana betting vault</p>
        </header>

        {!authenticated ? (
          <div className="flex flex-col items-center justify-center min-h-[60vh]">
            <WalletConnect />
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            <div className="lg:col-span-2 space-y-8">
              <VaultDashboard />
              <BetInterface />
              <BetHistory />
            </div>
            <div className="space-y-8">
              <div className="bg-gray-900 rounded-lg p-6">
                <h2 className="text-xl font-semibold mb-4">Account</h2>
                <WalletConnect />
              </div>
            </div>
          </div>
        )}
      </div>
    </main>
  );
}
