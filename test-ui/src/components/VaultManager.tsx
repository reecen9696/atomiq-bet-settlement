import { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { useConnection } from '@solana/wallet-adapter-react';
import { Building2, Loader2, ExternalLink } from 'lucide-react';
import { solanaService } from '../services/solana';
import { PublicKey } from '@solana/web3.js';

export function VaultManager() {
  const { publicKey, sendTransaction, signTransaction, signAllTransactions } = useWallet();
  const { connection } = useConnection();
  const [vaultAddress, setVaultAddress] = useState<string>('');
  const [casinoAddress, setCasinoAddress] = useState<string>('');
  const [vaultAuthorityAddress, setVaultAuthorityAddress] = useState<string>('');
  const [isCreating, setIsCreating] = useState(false);

  const [vaultExists, setVaultExists] = useState<boolean | null>(null);
  const [casinoExists, setCasinoExists] = useState<boolean | null>(null);

  const [casinoAuthority, setCasinoAuthority] = useState<string>('');
  const [casinoProcessor, setCasinoProcessor] = useState<string>('');
  const [casinoPaused, setCasinoPaused] = useState<boolean | null>(null);
  const [casinoTotalBets, setCasinoTotalBets] = useState<bigint | null>(null);
  const [casinoTotalVolumeLamports, setCasinoTotalVolumeLamports] = useState<bigint | null>(null);

  const [vaultLamports, setVaultLamports] = useState<number | null>(null);
  const [vaultTrackedLamports, setVaultTrackedLamports] = useState<bigint | null>(null);
  const [vaultLastActivity, setVaultLastActivity] = useState<bigint | null>(null);

  const [depositSol, setDepositSol] = useState('0.1');
  const [withdrawSol, setWithdrawSol] = useState('0.1');
  const [allowanceSol, setAllowanceSol] = useState('0.01');
  const [allowanceDuration, setAllowanceDuration] = useState('3600');
  const [lastAllowancePda, setLastAllowancePda] = useState<string>('');
  const [revokeAllowancePda, setRevokeAllowancePda] = useState<string>('');

  const [allowanceExists, setAllowanceExists] = useState<boolean | null>(null);
  const [allowanceAmountLamports, setAllowanceAmountLamports] = useState<bigint | null>(null);
  const [allowanceSpentLamports, setAllowanceSpentLamports] = useState<bigint | null>(null);
  const [allowanceRemainingLamports, setAllowanceRemainingLamports] = useState<bigint | null>(null);
  const [allowanceExpiresAt, setAllowanceExpiresAt] = useState<bigint | null>(null);
  const [allowanceRevoked, setAllowanceRevoked] = useState<boolean | null>(null);

  const [lastSignature, setLastSignature] = useState<string>('');
  const [statusMsg, setStatusMsg] = useState<string>('');
  const [errorMsg, setErrorMsg] = useState<string>('');
  const [errorDetails, setErrorDetails] = useState<string>('');

  const allowanceStorageKey = publicKey
    ? `atomik:lastAllowancePda:${publicKey.toBase58()}`
    : null;

  useEffect(() => {
    if (publicKey) {
      setErrorMsg('');
      setStatusMsg('');

      Promise.all([
        solanaService.deriveCasinoPDA(),
        solanaService.deriveVaultAuthorityPDA(),
        solanaService.deriveVaultPDA(publicKey.toBase58()),
      ])
        .then(async ([casino, vaultAuthority, vault]) => {
          setCasinoAddress(casino);
          setVaultAuthorityAddress(vaultAuthority);
          setVaultAddress(vault);

          const [casinoOk, vaultOk] = await Promise.all([
            solanaService.getAccountExists(casino, connection),
            solanaService.getAccountExists(vault, connection),
          ]);
          setCasinoExists(casinoOk);
          setVaultExists(vaultOk);

          if (casinoOk) {
            const casinoInfo = await solanaService.getCasinoInfoByAddress(casino, connection);
            setCasinoAuthority(casinoInfo.state?.authority ?? '');
            setCasinoProcessor(casinoInfo.state?.processor ?? '');
            setCasinoPaused(casinoInfo.state?.paused ?? null);
            setCasinoTotalBets(casinoInfo.state?.totalBets ?? null);
            setCasinoTotalVolumeLamports(casinoInfo.state?.totalVolumeLamports ?? null);
          } else {
            setCasinoAuthority('');
            setCasinoProcessor('');
            setCasinoPaused(null);
            setCasinoTotalBets(null);
            setCasinoTotalVolumeLamports(null);
          }

          if (vaultOk) {
            const info = await solanaService.getVaultInfoByAddress(vault, connection);
            setVaultLamports(info.lamports);
            setVaultTrackedLamports(info.state?.solBalanceLamports ?? null);
            setVaultLastActivity(info.state?.lastActivity ?? null);
          } else {
            setVaultLamports(null);
            setVaultTrackedLamports(null);
            setVaultLastActivity(null);
          }
        })
        .catch((err) => {
          console.error('Failed to derive PDAs:', err);
          setErrorMsg(err instanceof Error ? err.message : 'Failed to derive PDAs');
        });
    }
  }, [publicKey, connection]);

  useEffect(() => {
    if (!allowanceStorageKey) return;
    try {
      const saved = localStorage.getItem(allowanceStorageKey);
      if (saved && saved.length > 20) {
        setLastAllowancePda(saved);
        setRevokeAllowancePda((prev) => (prev ? prev : saved));
        refreshAllowanceInfo(saved);
      }
    } catch (err) {
      // ignore localStorage errors (private mode, etc.)
      console.warn('Unable to read saved allowance PDA:', err);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [allowanceStorageKey]);

  const refreshVaultInfo = async () => {
    if (!publicKey || !vaultAddress) return;
    try {
      const info = await solanaService.getVaultInfoByAddress(vaultAddress, connection);
      setVaultExists(info.exists);
      setVaultLamports(info.lamports);
      setVaultTrackedLamports(info.state?.solBalanceLamports ?? null);
      setVaultLastActivity(info.state?.lastActivity ?? null);
    } catch (err) {
      console.error('Failed to refresh vault info:', err);
    }
  };

  const refreshAllowanceInfo = async (allowanceAddress: string) => {
    if (!allowanceAddress) return;
    try {
      const info = await solanaService.getAllowanceInfoByAddress(allowanceAddress, connection);
      setAllowanceExists(info.exists);
      setAllowanceAmountLamports(info.state?.amountLamports ?? null);
      setAllowanceSpentLamports(info.state?.spentLamports ?? null);
      setAllowanceRemainingLamports(info.state?.remainingLamports ?? null);
      setAllowanceExpiresAt(info.state?.expiresAt ?? null);
      setAllowanceRevoked(info.state?.revoked ?? null);
    } catch (err) {
      console.error('Failed to refresh allowance info:', err);
      // Keep previous allowance values on transient RPC failures (e.g. 429).
    }
  };

  useEffect(() => {
    if (revokeAllowancePda && revokeAllowancePda.length > 20) {
      refreshAllowanceInfo(revokeAllowancePda);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [revokeAllowancePda]);

  const handleInitializeCasino = async () => {
    if (!publicKey) return;

    if (casinoExists === true) {
      setStatusMsg('Casino already initialized on-chain');
      setErrorMsg('');
      setErrorDetails('');
      return;
    }
    setIsCreating(true);
    setErrorMsg('');
    setErrorDetails('');
    setStatusMsg('Initializing casino...');

    try {
      const { signature, casinoPda, vaultAuthorityPda } = await solanaService.initializeCasinoVault({
        authority: publicKey,
        sendTransaction,
        signTransaction: signTransaction ?? undefined,
        connection,
      });
      setLastSignature(signature);
      setCasinoAddress(casinoPda);
      setVaultAuthorityAddress(vaultAuthorityPda);
      setCasinoExists(true);
      setStatusMsg('Casino initialized');
    } catch (err) {
      console.error('Failed to initialize casino:', err);
      const msg = err instanceof Error ? err.message : 'Failed to initialize casino';
      setErrorMsg(msg);
      setErrorDetails(
        typeof err === 'object' && err !== null
          ? JSON.stringify(err, Object.getOwnPropertyNames(err), 2)
          : String(err)
      );
      setStatusMsg('');
    } finally {
      setIsCreating(false);
    }
  };

  const handleInitializeVault = async () => {
    if (!publicKey) return;
    setIsCreating(true);
    setErrorMsg('');
    setErrorDetails('');
    setStatusMsg('Initializing user vault...');

    try {
      const { signature, vaultPda } = await solanaService.initializeUserVault({
        user: publicKey,
        sendTransaction,
        signTransaction: signTransaction ?? undefined,
        connection,
      });
      setLastSignature(signature);
      setVaultAddress(vaultPda);
      setVaultExists(true);
      await refreshVaultInfo();
      setStatusMsg('Vault initialized');
    } catch (err) {
      console.error('Failed to initialize vault:', err);
      const msg = err instanceof Error ? err.message : 'Failed to initialize vault';
      setErrorMsg(msg);
      setErrorDetails(
        typeof err === 'object' && err !== null
          ? JSON.stringify(err, Object.getOwnPropertyNames(err), 2)
          : String(err)
      );
      setStatusMsg('');
    } finally {
      setIsCreating(false);
    }
  };

  const handleDeposit = async () => {
    if (!publicKey) return;
    setIsCreating(true);
    setErrorMsg('');
    setErrorDetails('');
    setStatusMsg('Depositing SOL to vault...');
    try {
      const deposit = Number(depositSol);
      if (!Number.isFinite(deposit) || deposit <= 0) throw new Error('Enter a valid deposit amount');
      const amount = BigInt(Math.floor(deposit * 1_000_000_000));
      const { signature } = await solanaService.depositSol({
        user: publicKey,
        amountLamports: amount,
        sendTransaction,
        signTransaction: signTransaction ?? undefined,
        connection,
      });
      setLastSignature(signature);
      await refreshVaultInfo();
      setStatusMsg('Deposit confirmed');
    } catch (err) {
      console.error('Deposit failed:', err);
      const msg = err instanceof Error ? err.message : 'Deposit failed';
      setErrorMsg(msg);
      setErrorDetails(
        typeof err === 'object' && err !== null
          ? JSON.stringify(err, Object.getOwnPropertyNames(err), 2)
          : String(err)
      );
      setStatusMsg('');
    } finally {
      setIsCreating(false);
    }
  };

  const handleWithdraw = async () => {
    if (!publicKey) return;
    setIsCreating(true);
    setErrorMsg('');
    setErrorDetails('');
    setStatusMsg('Withdrawing SOL from vault...');
    try {
      const withdraw = Number(withdrawSol);
      if (!Number.isFinite(withdraw) || withdraw <= 0) throw new Error('Enter a valid withdraw amount');
      const amount = BigInt(Math.floor(withdraw * 1_000_000_000));
      const { signature } = await solanaService.withdrawSol({
        user: publicKey,
        amountLamports: amount,
        sendTransaction,
        signTransaction: signTransaction ?? undefined,
        connection,
      });
      setLastSignature(signature);
      await refreshVaultInfo();
      setStatusMsg('Withdraw confirmed');
    } catch (err) {
      console.error('Withdraw failed:', err);
      const msg = err instanceof Error ? err.message : 'Withdraw failed';
      setErrorMsg(msg);
      setErrorDetails(
        typeof err === 'object' && err !== null
          ? JSON.stringify(err, Object.getOwnPropertyNames(err), 2)
          : String(err)
      );
      setStatusMsg('');
    } finally {
      setIsCreating(false);
    }
  };

  const handleApproveAllowance = async () => {
    if (!publicKey) return;
    setIsCreating(true);
    setErrorMsg('');
    setErrorDetails('');
    setStatusMsg('Approving allowance...');
    try {
      const allowance = Number(allowanceSol);
      const durationSec = Number(allowanceDuration);
      if (!Number.isFinite(allowance) || allowance <= 0) throw new Error('Enter a valid allowance amount');
      if (!Number.isFinite(durationSec) || durationSec <= 0) throw new Error('Enter a valid allowance duration');
      const amount = BigInt(Math.floor(allowance * 1_000_000_000));
      const duration = BigInt(Math.floor(durationSec));
      const { signature, allowancePda } = await solanaService.approveAllowanceSol({
        user: publicKey,
        amountLamports: amount,
        durationSeconds: duration,
        sendTransaction,
        signTransaction: signTransaction ?? undefined,
        signAllTransactions: signAllTransactions ?? undefined,
        connection,
      });
      setLastSignature(signature);
      setLastAllowancePda(allowancePda);
      setRevokeAllowancePda(allowancePda);

      if (allowanceStorageKey) {
        try {
          localStorage.setItem(allowanceStorageKey, allowancePda);
        } catch (err) {
          console.warn('Unable to persist allowance PDA:', err);
        }
      }

      await refreshAllowanceInfo(allowancePda);
      setStatusMsg('Allowance approved');
    } catch (err) {
      console.error('Approve allowance failed:', err);
      const rawMsg = err instanceof Error ? err.message : String(err);
      const msg =
        rawMsg.includes('code": 429') || rawMsg.includes(' 429') || rawMsg.toLowerCase().includes('too many requests')
          ? 'RPC rate-limited (429). Public devnet RPC is throttling you — set VITE_SOLANA_RPC_URL to a higher-limit endpoint and retry.'
          : rawMsg || 'Approve allowance failed';
      setErrorMsg(msg);
      setErrorDetails(
        typeof err === 'object' && err !== null
          ? JSON.stringify(err, Object.getOwnPropertyNames(err), 2)
          : String(err)
      );
      setStatusMsg('');
    } finally {
      setIsCreating(false);
    }
  };

  const handleRevokeAllowance = async () => {
    if (!publicKey) return;
    if (!revokeAllowancePda) {
      setErrorMsg('Enter an allowance address to revoke');
      setErrorDetails('');
      return;
    }
    setIsCreating(true);
    setErrorMsg('');
    setErrorDetails('');
    setStatusMsg('Revoking allowance...');
    try {
      const { signature } = await solanaService.revokeAllowance({
        user: publicKey,
        allowancePda: new PublicKey(revokeAllowancePda),
        sendTransaction,
        signTransaction: signTransaction ?? undefined,
        connection,
      });
      setLastSignature(signature);
      setStatusMsg('Allowance revoked');
    } catch (err) {
      console.error('Revoke allowance failed:', err);
      const msg = err instanceof Error ? err.message : 'Revoke allowance failed';
      setErrorMsg(msg);
      setErrorDetails(
        typeof err === 'object' && err !== null
          ? JSON.stringify(err, Object.getOwnPropertyNames(err), 2)
          : String(err)
      );
      setStatusMsg('');
    } finally {
      setIsCreating(false);
    }
  };

  if (!publicKey) {
    return (
      <div className="p-6 bg-gray-50 rounded-lg border border-gray-200">
        <div className="flex items-center text-gray-500">
          <Building2 className="w-5 h-5 mr-2" />
          <span>Connect wallet first to manage vault</span>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow-lg border border-gray-200">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold text-gray-800 flex items-center">
          <Building2 className="w-6 h-6 mr-2 text-purple-500" />
          Vault Management
        </h2>
      </div>

      <div className="space-y-4">
        {errorMsg && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-sm text-red-800">{errorMsg}</p>
            {errorDetails && (
              <details className="mt-2">
                <summary className="text-xs text-red-700 cursor-pointer select-none">
                  Show details
                </summary>
                <pre className="mt-2 text-xs text-red-900 whitespace-pre-wrap break-words bg-white border border-red-200 rounded p-2 max-h-64 overflow-auto">
                  {errorDetails}
                </pre>
              </details>
            )}
          </div>
        )}

        {statusMsg && (
          <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
            <p className="text-sm text-blue-800">{statusMsg}</p>
          </div>
        )}

        {lastSignature && (
          <div className="p-3 bg-gray-50 rounded-lg border border-gray-200">
            <p className="text-xs text-gray-500 mb-1">Last Transaction</p>
            <div className="flex items-center justify-between bg-white p-2 rounded">
              <code className="text-xs text-gray-800 truncate mr-2">{lastSignature}</code>
              <a
                href={solanaService.getExplorerUrl(lastSignature)}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-purple-600 hover:underline inline-flex items-center"
              >
                View
                <ExternalLink className="w-3 h-3 ml-1" />
              </a>
            </div>
          </div>
        )}

        <div className="p-4 bg-white rounded-lg border border-gray-200">
          <p className="text-sm text-gray-600 mb-2 font-semibold">Casino PDA</p>
          {casinoAddress ? (
            <>
              <code className="text-xs text-gray-800 break-all block bg-gray-50 p-2 rounded">
                {casinoAddress}
              </code>
              <div className="flex items-center justify-between mt-2">
                <span className={`text-xs font-semibold ${casinoExists ? 'text-green-700' : 'text-yellow-700'}`}>
                  {casinoExists === null ? 'Checking…' : casinoExists ? 'Exists on-chain' : 'Not initialized'}
                </span>
                <a
                  href={solanaService.getAccountExplorerUrl(casinoAddress)}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs text-purple-600 hover:underline inline-flex items-center"
                >
                  Explorer
                  <ExternalLink className="w-3 h-3 ml-1" />
                </a>
              </div>

              {casinoExists && (casinoAuthority || casinoProcessor || casinoPaused !== null) && (
                <div className="mt-3 p-3 bg-white rounded border border-gray-200">
                  <p className="text-xs font-semibold text-gray-700 mb-2">Casino State (on-chain)</p>
                  <div className="space-y-1 text-xs">
                    {casinoAuthority && (
                      <div className="flex items-center justify-between gap-2">
                        <span className="text-gray-500">Authority:</span>
                        <span className="font-mono text-gray-800 break-all text-right">{casinoAuthority}</span>
                      </div>
                    )}
                    {casinoProcessor && (
                      <div className="flex items-center justify-between gap-2">
                        <span className="text-gray-500">Processor:</span>
                        <span className="font-mono text-gray-800 break-all text-right">{casinoProcessor}</span>
                      </div>
                    )}
                    <div className="flex items-center justify-between">
                      <span className="text-gray-500">Paused:</span>
                      <span className="font-semibold text-gray-800">
                        {casinoPaused === null ? '—' : casinoPaused ? 'Yes' : 'No'}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-500">Total bets:</span>
                      <span className="font-mono text-gray-800">
                        {casinoTotalBets === null ? '—' : casinoTotalBets.toString()}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-500">Total volume:</span>
                      <span className="font-mono text-gray-800">
                        {casinoTotalVolumeLamports === null
                          ? '—'
                          : `${(Number(casinoTotalVolumeLamports) / 1_000_000_000).toFixed(6)} SOL`}
                      </span>
                    </div>
                  </div>
                  <p className="mt-2 text-[11px] text-gray-500">
                    For full E2E bets, the running processor service must use the same “Processor” pubkey.
                  </p>
                </div>
              )}
            </>
          ) : (
            <div className="flex items-center text-gray-500">
              <Loader2 className="w-4 h-4 animate-spin mr-2" />
              <span className="text-sm">Deriving casino PDA…</span>
            </div>
          )}

          <p className="text-sm text-gray-600 mb-2 mt-4 font-semibold">Vault Authority PDA</p>
          {vaultAuthorityAddress ? (
            <>
              <code className="text-xs text-gray-800 break-all block bg-gray-50 p-2 rounded">
                {vaultAuthorityAddress}
              </code>
              <a
                href={solanaService.getAccountExplorerUrl(vaultAuthorityAddress)}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-purple-600 hover:underline mt-2 inline-flex items-center"
              >
                Explorer
                <ExternalLink className="w-3 h-3 ml-1" />
              </a>
            </>
          ) : (
            <div className="flex items-center text-gray-500">
              <Loader2 className="w-4 h-4 animate-spin mr-2" />
              <span className="text-sm">Deriving vault authority PDA…</span>
            </div>
          )}

          <button
            onClick={handleInitializeCasino}
            disabled={isCreating || !casinoAddress || casinoExists === true}
            className="w-full mt-4 bg-gradient-to-r from-indigo-500 to-purple-600 text-white px-6 py-3 rounded-lg hover:from-indigo-600 hover:to-purple-700 disabled:from-gray-400 disabled:to-gray-500 disabled:cursor-not-allowed transition-all duration-200 font-semibold shadow-md hover:shadow-lg flex items-center justify-center"
          >
            {isCreating ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin mr-2" />
                Working...
              </>
            ) : (
              casinoExists === true ? 'Casino Already Initialized' : 'Initialize Casino (Admin)'
            )}
          </button>
        </div>

        <div className="p-4 bg-gradient-to-r from-purple-50 to-pink-50 rounded-lg border border-purple-200">
          <p className="text-sm text-gray-600 mb-2 font-semibold">Your Vault Address (PDA)</p>
          {vaultAddress ? (
            <>
              <code className="text-xs text-gray-800 break-all block bg-white p-2 rounded">
                {vaultAddress}
              </code>
              <div className="flex items-center justify-between mt-2">
                <span className={`text-xs font-semibold ${vaultExists ? 'text-green-700' : 'text-yellow-700'}`}>
                  {vaultExists === null ? 'Checking…' : vaultExists ? 'Exists on-chain' : 'Not initialized'}
                </span>
                <a
                  href={solanaService.getAccountExplorerUrl(vaultAddress)}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs text-purple-600 hover:underline inline-flex items-center"
                >
                  Explorer
                  <ExternalLink className="w-3 h-3 ml-1" />
                </a>
              </div>

              {vaultExists && (
                <div className="mt-3 p-3 bg-white rounded border border-purple-200">
                  <div className="flex items-center justify-between">
                    <p className="text-xs font-semibold text-gray-700">Vault Balance (on-chain)</p>
                    <button
                      onClick={refreshVaultInfo}
                      className="text-xs text-purple-600 hover:underline"
                      type="button"
                    >
                      Refresh
                    </button>
                  </div>
                  <div className="mt-2 grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                    <div className="flex items-center justify-between">
                      <span className="text-gray-500">Account lamports:</span>
                      <span className="font-mono text-gray-800">
                        {vaultLamports === null ? '—' : `${(vaultLamports / 1_000_000_000).toFixed(6)} SOL`}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-500">Tracked sol_balance:</span>
                      <span className="font-mono text-gray-800">
                        {vaultTrackedLamports === null
                          ? '—'
                          : `${(Number(vaultTrackedLamports) / 1_000_000_000).toFixed(6)} SOL`}
                      </span>
                    </div>
                    <div className="flex items-center justify-between md:col-span-2">
                      <span className="text-gray-500">Last activity:</span>
                      <span className="text-gray-800">
                        {vaultLastActivity === null
                          ? '—'
                          : new Date(Number(vaultLastActivity) * 1000).toLocaleString()}
                      </span>
                    </div>
                  </div>
                  <p className="mt-2 text-[11px] text-gray-500">
                    “Account lamports” includes rent-exempt reserve; “Tracked sol_balance” is the program’s internal balance.
                  </p>
                </div>
              )}
            </>
          ) : (
            <div className="flex items-center text-gray-500">
              <Loader2 className="w-4 h-4 animate-spin mr-2" />
              <span className="text-sm">Deriving vault address...</span>
            </div>
          )}
        </div>

        <div className="p-4 bg-blue-50 rounded-lg border border-blue-200">
          <p className="text-sm text-blue-800 mb-3">
            Initializes your user vault PDA on-chain (requires casino to be initialized once).
          </p>
          <button
            onClick={handleInitializeVault}
            disabled={isCreating || !vaultAddress || vaultExists === true}
            className="w-full bg-gradient-to-r from-purple-500 to-pink-600 text-white px-6 py-3 rounded-lg hover:from-purple-600 hover:to-pink-700 disabled:from-gray-400 disabled:to-gray-500 disabled:cursor-not-allowed transition-all duration-200 font-semibold shadow-md hover:shadow-lg flex items-center justify-center"
          >
            {isCreating ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin mr-2" />
                Working...
              </>
            ) : (
              vaultExists === true ? 'Vault Already Initialized' : 'Initialize Vault'
            )}
          </button>
        </div>

        <div className="p-4 bg-green-50 rounded-lg border border-green-200">
          <p className="text-sm font-semibold text-green-900 mb-3">Deposit / Withdraw SOL</p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <label className="block text-xs text-green-900 mb-1">Deposit (SOL)</label>
              <input
                type="number"
                min="0"
                step="0.01"
                value={depositSol}
                onChange={(e) => setDepositSol(e.target.value)}
                className="w-full px-3 py-2 rounded border border-green-200 bg-white text-sm font-mono"
              />
              <button
                onClick={handleDeposit}
                disabled={isCreating}
                className="w-full mt-2 bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-sm font-semibold"
              >
                Deposit
              </button>
            </div>
            <div>
              <label className="block text-xs text-green-900 mb-1">Withdraw (SOL)</label>
              <input
                type="number"
                min="0"
                step="0.01"
                value={withdrawSol}
                onChange={(e) => setWithdrawSol(e.target.value)}
                className="w-full px-3 py-2 rounded border border-green-200 bg-white text-sm font-mono"
              />
              <button
                onClick={handleWithdraw}
                disabled={isCreating}
                className="w-full mt-2 bg-emerald-600 text-white px-4 py-2 rounded hover:bg-emerald-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-sm font-semibold"
              >
                Withdraw
              </button>
            </div>
          </div>
          <p className="text-xs text-green-900 mt-3">
            These calls require your vault to be initialized.
          </p>
        </div>

        <div className="p-4 bg-orange-50 rounded-lg border border-orange-200">
          <p className="text-sm font-semibold text-orange-900 mb-3">Allowance (SOL)</p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <label className="block text-xs text-orange-900 mb-1">Amount (SOL)</label>
              <input
                type="number"
                min="0"
                step="0.01"
                value={allowanceSol}
                onChange={(e) => setAllowanceSol(e.target.value)}
                className="w-full px-3 py-2 rounded border border-orange-200 bg-white text-sm font-mono"
              />
            </div>
            <div>
              <label className="block text-xs text-orange-900 mb-1">Duration (seconds)</label>
              <input
                type="number"
                min="1"
                step="1"
                value={allowanceDuration}
                onChange={(e) => setAllowanceDuration(e.target.value)}
                className="w-full px-3 py-2 rounded border border-orange-200 bg-white text-sm font-mono"
              />
            </div>
          </div>
          <button
            onClick={handleApproveAllowance}
            disabled={isCreating}
            className="w-full mt-3 bg-orange-600 text-white px-4 py-2 rounded hover:bg-orange-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-sm font-semibold"
          >
            Approve Allowance
          </button>

          {lastAllowancePda && (
            <div className="mt-3">
              <p className="text-xs text-orange-900 mb-1">Last allowance PDA</p>
              <code className="text-xs text-gray-800 break-all block bg-white p-2 rounded border border-orange-200">
                {lastAllowancePda}
              </code>
            </div>
          )}

          {(allowanceExists !== null || allowanceAmountLamports !== null) && (
            <div className="mt-3 p-3 bg-white rounded border border-orange-200">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold text-gray-700">Current SOL Allowance (on-chain)</p>
                <button
                  type="button"
                  className="text-xs text-orange-700 hover:underline"
                  onClick={() => refreshAllowanceInfo(revokeAllowancePda || lastAllowancePda)}
                >
                  Refresh
                </button>
              </div>
              <div className="mt-2 grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Exists:</span>
                  <span className={`font-semibold ${allowanceExists ? 'text-green-700' : 'text-yellow-700'}`}>
                    {allowanceExists === null ? '—' : allowanceExists ? 'Yes' : 'No'}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Revoked:</span>
                  <span className="font-semibold text-gray-800">
                    {allowanceRevoked === null ? '—' : allowanceRevoked ? 'Yes' : 'No'}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Amount:</span>
                  <span className="font-mono text-gray-800">
                    {allowanceAmountLamports === null
                      ? '—'
                      : `${(Number(allowanceAmountLamports) / 1_000_000_000).toFixed(6)} SOL`}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Spent:</span>
                  <span className="font-mono text-gray-800">
                    {allowanceSpentLamports === null
                      ? '—'
                      : `${(Number(allowanceSpentLamports) / 1_000_000_000).toFixed(6)} SOL`}
                  </span>
                </div>
                <div className="flex items-center justify-between md:col-span-2">
                  <span className="text-gray-500">Remaining:</span>
                  <span className="font-mono text-gray-800">
                    {allowanceRemainingLamports === null
                      ? '—'
                      : `${(Number(allowanceRemainingLamports) / 1_000_000_000).toFixed(6)} SOL`}
                  </span>
                </div>
                <div className="flex items-center justify-between md:col-span-2">
                  <span className="text-gray-500">Expires:</span>
                  <span className="text-gray-800">
                    {allowanceExpiresAt === null
                      ? '—'
                      : new Date(Number(allowanceExpiresAt) * 1000).toLocaleString()}
                  </span>
                </div>
              </div>
              <p className="mt-2 text-[11px] text-gray-500">
                Note: every approval creates a new allowance PDA; paste any allowance PDA below to inspect it.
              </p>
            </div>
          )}

          <div className="mt-4">
            <label className="block text-xs text-orange-900 mb-1">Allowance PDA to revoke</label>
            <input
              type="text"
              value={revokeAllowancePda}
              onChange={(e) => setRevokeAllowancePda(e.target.value)}
              placeholder="Allowance address"
              className="w-full px-3 py-2 rounded border border-orange-200 bg-white text-sm font-mono"
            />
            <button
              onClick={handleRevokeAllowance}
              disabled={isCreating}
              className="w-full mt-2 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-sm font-semibold"
            >
              Revoke Allowance
            </button>
          </div>
        </div>

        <div className="p-4 bg-yellow-50 rounded-lg border border-yellow-200">
          <div className="text-xs text-yellow-800">
            <p>
              💡 <strong>Getting Started:</strong>
            </p>
            <ol className="list-decimal list-inside mt-2 space-y-1">
              <li>Request an airdrop above to get devnet SOL</li>
              <li>Initialize casino once (admin)</li>
              <li>Initialize your vault, then deposit SOL</li>
              <li>Optionally approve an allowance for the processor to spend</li>
            </ol>
          </div>
        </div>
      </div>
    </div>
  );
}