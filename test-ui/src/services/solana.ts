import {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionMessage,
  TransactionInstruction,
  type TransactionSignature,
  VersionedTransaction,
} from '@solana/web3.js';
import type { SendTransactionOptions } from '@solana/wallet-adapter-base';

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || 'https://api.devnet.solana.com';
const VAULT_PROGRAM_ID = import.meta.env.VITE_VAULT_PROGRAM_ID;
const SOLANA_NETWORK = (import.meta.env.VITE_SOLANA_NETWORK || 'devnet') as string;

type SendTransactionFn = (
  transaction: Transaction | VersionedTransaction,
  connection: Connection,
  options?: SendTransactionOptions
) => Promise<TransactionSignature>;

type SignTransactionFn = (
  transaction: Transaction | VersionedTransaction
) => Promise<Transaction | VersionedTransaction>;

type SignAllTransactionsFn = (
  transactions: (Transaction | VersionedTransaction)[]
) => Promise<(Transaction | VersionedTransaction)[]>;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function isRateLimitError(err: unknown): boolean {
  const anyErr = err as any;
  const msg = err instanceof Error ? err.message : String(err);
  return (
    anyErr?.code === 429 ||
    msg.includes(' 429') ||
    msg.includes('code": 429') ||
    msg.toLowerCase().includes('too many requests') ||
    msg.toLowerCase().includes('rate limit')
  );
}

// Public Solana RPC endpoints (especially devnet) are aggressively rate-limited.
// web3.js will also retry some 429s internally; if we *also* retry without
// coordination, we can create a thundering herd. This helper:
// - serializes our own RPC calls (concurrency=1)
// - applies a global cooldown when 429s occur
let rpcInFlight = 0;
const rpcQueue: Array<() => void> = [];
let rpcCooldownUntilMs = 0;

async function acquireRpcSlot(): Promise<void> {
  if (rpcInFlight === 0) {
    rpcInFlight = 1;
    return;
  }
  await new Promise<void>((resolve) => rpcQueue.push(resolve));
  rpcInFlight = 1;
}

function releaseRpcSlot(): void {
  rpcInFlight = 0;
  const next = rpcQueue.shift();
  if (next) next();
}

async function withRateLimitRetry<T>(fn: () => Promise<T>, opts?: { retries?: number; baseDelayMs?: number }): Promise<T> {
  const retries = opts?.retries ?? 2;
  const baseDelayMs = opts?.baseDelayMs ?? 1000;
  let attempt = 0;

  while (true) {
    const now = Date.now();
    if (rpcCooldownUntilMs > now) {
      await sleep(rpcCooldownUntilMs - now);
    }

    await acquireRpcSlot();
    try {
      return await fn();
    } catch (err) {
      if (!isRateLimitError(err) || attempt >= retries) throw err;

      const delay = baseDelayMs * Math.pow(2, attempt);
      // Global cooldown so other calls back off too.
      rpcCooldownUntilMs = Math.max(rpcCooldownUntilMs, Date.now() + delay);
      attempt += 1;
      // Small jitter to reduce synchronization.
      await sleep(delay + Math.floor(Math.random() * 250));
    } finally {
      releaseRpcSlot();
    }
  }
}

export type VaultAccountState = {
  owner: string;
  casino: string;
  bump: number;
  solBalanceLamports: bigint;
  createdAt: bigint;
  lastActivity: bigint;
};

export type AllowanceAccountState = {
  user: string;
  casino: string;
  tokenMint: string;
  amountLamports: bigint;
  spentLamports: bigint;
  expiresAt: bigint;
  createdAt: bigint;
  nonce: bigint;
  revoked: boolean;
  bump: number;
  lastSpentAt: bigint;
  spendCount: number;
  remainingLamports: bigint;
};

export type AllowanceNonceRegistryState = {
  user: string;
  casino: string;
  nextNonce: bigint;
  bump: number;
};

function readPubkey(buf: Buffer, offset: number): PublicKey {
  return new PublicKey(buf.subarray(offset, offset + 32));
}

function parseVaultAccount(data: Buffer): VaultAccountState {
  // Anchor account layout: 8-byte discriminator + fields.
  // Vault fields: owner(32) casino(32) bump(u8) sol_balance(u64) created_at(i64) last_activity(i64)
  if (data.length < 8 + 32 + 32 + 1 + 8 + 8 + 8) {
    throw new Error(`Vault account data too small: ${data.length} bytes`);
  }
  let off = 8;
  const owner = readPubkey(data, off);
  off += 32;
  const casino = readPubkey(data, off);
  off += 32;
  const bump = data.readUInt8(off);
  off += 1;
  const solBalanceLamports = data.readBigUInt64LE(off);
  off += 8;
  const createdAt = data.readBigInt64LE(off);
  off += 8;
  const lastActivity = data.readBigInt64LE(off);

  return {
    owner: owner.toBase58(),
    casino: casino.toBase58(),
    bump,
    solBalanceLamports,
    createdAt,
    lastActivity,
  };
}

function parseAllowanceAccount(data: Buffer): AllowanceAccountState {
  // Anchor account layout: 8-byte discriminator + fields.
  // Allowance fields:
  // user(32) casino(32) token_mint(32) amount(u64) spent(u64) expires_at(i64)
  // created_at(i64) nonce(u64) revoked(bool) bump(u8) last_spent_at(i64) spend_count(u32)
  const minLen = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 8 + 4;
  if (data.length < minLen) {
    throw new Error(`Allowance account data too small: ${data.length} bytes`);
  }

  let off = 8;
  const user = readPubkey(data, off);
  off += 32;
  const casino = readPubkey(data, off);
  off += 32;
  const tokenMint = readPubkey(data, off);
  off += 32;
  const amountLamports = data.readBigUInt64LE(off);
  off += 8;
  const spentLamports = data.readBigUInt64LE(off);
  off += 8;
  const expiresAt = data.readBigInt64LE(off);
  off += 8;
  const createdAt = data.readBigInt64LE(off);
  off += 8;
  const nonce = data.readBigUInt64LE(off);
  off += 8;
  const revoked = data.readUInt8(off) !== 0;
  off += 1;
  const bump = data.readUInt8(off);
  off += 1;
  const lastSpentAt = data.readBigInt64LE(off);
  off += 8;
  const spendCount = data.readUInt32LE(off);

  const remainingLamports = amountLamports > spentLamports ? amountLamports - spentLamports : 0n;

  return {
    user: user.toBase58(),
    casino: casino.toBase58(),
    tokenMint: tokenMint.toBase58(),
    amountLamports,
    spentLamports,
    expiresAt,
    createdAt,
    nonce,
    revoked,
    bump,
    lastSpentAt,
    spendCount,
    remainingLamports,
  };
}

function parseAllowanceNonceRegistryAccount(data: Buffer): AllowanceNonceRegistryState {
  // Anchor account layout: 8-byte discriminator + fields.
  // AllowanceNonceRegistry fields: user(32) casino(32) next_nonce(u64) bump(u8)
  const minLen = 8 + 32 + 32 + 8 + 1;
  if (data.length < minLen) {
    throw new Error(`AllowanceNonceRegistry account data too small: ${data.length} bytes`);
  }

  let off = 8;
  const user = readPubkey(data, off);
  off += 32;
  const casino = readPubkey(data, off);
  off += 32;
  const nextNonce = data.readBigUInt64LE(off);
  off += 8;
  const bump = data.readUInt8(off);

  return {
    user: user.toBase58(),
    casino: casino.toBase58(),
    nextNonce,
    bump,
  };
}

function i64ToLeBytes(value: bigint): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigInt64LE(value);
  return buf;
}

function u64ToLeBytes(value: bigint): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(value);
  return buf;
}

async function anchorDiscriminator(ixName: string): Promise<Buffer> {
  // Anchor: first 8 bytes of sha256("global:<ix_name>")
  const preimage = new TextEncoder().encode(`global:${ixName}`);
  if (!globalThis.crypto?.subtle?.digest) {
    throw new Error('WebCrypto not available: cannot compute Anchor discriminator');
  }
  const bytes: ArrayBuffer = preimage.buffer.slice(
    preimage.byteOffset,
    preimage.byteOffset + preimage.byteLength
  ) as ArrayBuffer;
  const hash = await globalThis.crypto.subtle.digest('SHA-256', bytes);
  return Buffer.from(hash).subarray(0, 8);
}

async function buildIxData(ixName: string, args?: Buffer[]): Promise<Buffer> {
  const disc = await anchorDiscriminator(ixName);
  return Buffer.concat([disc, ...(args || [])]);
}

function isUserRejectedError(err: unknown): boolean {
  const msg = err instanceof Error ? err.message : String(err);
  return (
    msg.toLowerCase().includes('user rejected') ||
    msg.toLowerCase().includes('user declined') ||
    msg.toLowerCase().includes('rejected the request') ||
    msg.toLowerCase().includes('request rejected') ||
    msg.toLowerCase().includes('denied')
  );
}

function extractCustomProgramErrorCode(err: unknown): number | null {
  const anyErr = err as any;

  // Prefer structured fields we attach.
  const structuredCandidates: unknown[] = [anyErr?.statusErr, anyErr?.status?.err, anyErr?.simErr].filter(Boolean);
  for (const candidate of structuredCandidates) {
    const c: any = candidate as any;

    // Shape: { InstructionError: [ixIndex, { Custom: n }] }
    if (c && typeof c === 'object' && 'InstructionError' in c) {
      const ie = (c as any).InstructionError;
      if (Array.isArray(ie) && ie.length >= 2) {
        const detail = ie[1];
        if (detail && typeof detail === 'object' && 'Custom' in (detail as any)) {
          const code = (detail as any).Custom;
          if (typeof code === 'number' && Number.isFinite(code)) return code;
        }
      }
    }

    // Some RPCs return InstructionError as a tuple-like array at the top-level.
    if (Array.isArray(c) && c.length >= 2) {
      const detail = c[1];
      if (detail && typeof detail === 'object' && 'Custom' in (detail as any)) {
        const code = (detail as any).Custom;
        if (typeof code === 'number' && Number.isFinite(code)) return code;
      }
    }
  }

  // Fallback: parse from message text.
  const msg = err instanceof Error ? err.message : String(err);
  const hexMatch = msg.match(/custom program error:\s*0x([0-9a-fA-F]+)/);
  if (hexMatch?.[1]) {
    const parsed = Number.parseInt(hexMatch[1], 16);
    if (Number.isFinite(parsed)) return parsed;
  }
  const decMatch = msg.match(/"Custom"\s*:\s*(\d+)/);
  if (decMatch?.[1]) {
    const parsed = Number.parseInt(decMatch[1], 10);
    if (Number.isFinite(parsed)) return parsed;
  }
  return null;
}

async function confirmSignatureRobust(
  connection: Connection,
  args: { signature: string; blockhash: string; lastValidBlockHeight: number },
  commitment: 'processed' | 'confirmed' | 'finalized' = 'confirmed',
  opts?: { timeoutMs?: number; pollIntervalMs?: number }
): Promise<void> {
  // NOTE: Using confirmTransaction with lastValidBlockHeight can produce
  // TransactionExpiredBlockheightExceededError under slow RPC / backoff.
  // For UX, polling signature statuses is more reliable.
  const timeoutMs = opts?.timeoutMs ?? 60_000;
  const pollIntervalMs = opts?.pollIntervalMs ?? 1_250;

  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const statuses = await withRateLimitRetry(() =>
      connection.getSignatureStatuses([args.signature], { searchTransactionHistory: true })
    );
    const status = statuses.value[0];
    if (status) {
      if (status.err) {
        const wrapped = new Error(`Transaction failed: ${JSON.stringify(status.err)}`);
        (wrapped as any).signature = args.signature;
        (wrapped as any).status = status;
        (wrapped as any).statusErr = status.err;
        (wrapped as any).customErrorCode = extractCustomProgramErrorCode(wrapped);
        throw wrapped;
      }

      if (commitment === 'processed') return;
      if (commitment === 'confirmed') {
        if (status.confirmationStatus === 'confirmed' || status.confirmationStatus === 'finalized') return;
      } else {
        if (status.confirmationStatus === 'finalized') return;
      }
    }

    await sleep(pollIntervalMs);
  }

  const timeoutErr = new Error(`Confirmation timed out after ${Math.round(timeoutMs / 1000)}s for signature ${args.signature}`);
  (timeoutErr as any).signature = args.signature;
  throw timeoutErr;
}

async function signSendAndConfirm(
  connection: Connection,
  signTransaction: SignTransactionFn,
  tx: Transaction | VersionedTransaction
): Promise<string> {
  const latest = await withRateLimitRetry(() => connection.getLatestBlockhash('confirmed'));
  if (tx instanceof Transaction) {
    tx.recentBlockhash = latest.blockhash;
  }

  const signed = await signTransaction(tx);
  const raw = signed.serialize();
  const sig = await withRateLimitRetry(() =>
    connection.sendRawTransaction(raw, {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
      maxRetries: 3,
    })
  );

  await confirmSignatureRobust(
    connection,
    {
      signature: sig,
      blockhash: latest.blockhash,
      lastValidBlockHeight: latest.lastValidBlockHeight,
    },
    'confirmed'
  );

  return sig;
}

async function sendAndConfirm(
  connection: Connection,
  sendTransaction: SendTransactionFn,
  tx: Transaction | VersionedTransaction,
  opts?: { signTransaction?: SignTransactionFn }
): Promise<string> {
  const latest = await withRateLimitRetry(() => connection.getLatestBlockhash('confirmed'));
  if (tx instanceof Transaction) {
    tx.recentBlockhash = latest.blockhash;
  }
  try {
    const sig = await sendTransaction(tx, connection, {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
      maxRetries: 3,
    });

    await confirmSignatureRobust(
      connection,
      {
        signature: sig,
        blockhash: latest.blockhash,
        lastValidBlockHeight: latest.lastValidBlockHeight,
      },
      'confirmed'
    );
    return sig;
  } catch (err) {
    // Some wallets (or browser extensions) occasionally throw a generic
    // WalletSendTransactionError: Unexpected error. If we have a signTransaction
    // function available, fall back to signing + sending the raw transaction
    // ourselves to get a more actionable RPC error.
    if (opts?.signTransaction) {
      try {
        return await signSendAndConfirm(connection, opts.signTransaction, tx);
      } catch (fallbackErr) {
        // Keep original error as the primary cause, but attach fallback failure.
        const wrapped = new Error(
          `Wallet send failed, and raw-send fallback also failed: ${fallbackErr instanceof Error ? fallbackErr.message : String(fallbackErr)}`
        );
        (wrapped as any).cause = err;
        (wrapped as any).fallbackCause = fallbackErr;
        throw wrapped;
      }
    }

    // Attempt to surface useful program logs by simulating the transaction.
    try {
      const simLatest = await withRateLimitRetry(() => connection.getLatestBlockhash('confirmed'));
      // We do not verify signatures here; we only want logs.
      // web3.js has separate overloads for Transaction vs VersionedTransaction.
      // To consistently force sigVerify=false, simulate a VersionedTransaction.
      let simTx: VersionedTransaction;
      if (tx instanceof VersionedTransaction) {
        simTx = tx;
      } else {
        if (!tx.feePayer) {
          throw new Error('Cannot simulate legacy transaction without feePayer');
        }
        const msg = new TransactionMessage({
          payerKey: tx.feePayer,
          recentBlockhash: simLatest.blockhash,
          instructions: tx.instructions,
        }).compileToV0Message();
        simTx = new VersionedTransaction(msg);
      }

      const sim = await withRateLimitRetry(() =>
        connection.simulateTransaction(simTx, {
          commitment: 'confirmed',
          sigVerify: false,
          replaceRecentBlockhash: true,
        })
      );
      const logs = sim.value.logs || [];
      const errMsg = (sim.value.err ? JSON.stringify(sim.value.err) : null) || (err instanceof Error ? err.message : String(err));
      const wrapped = new Error(
        `Transaction failed during simulation: ${errMsg}${logs.length ? `\n\nProgram logs:\n${logs.join('\n')}` : ''}`
      );
      (wrapped as any).cause = err;
      (wrapped as any).logs = logs;
      (wrapped as any).simErr = sim.value.err;
      throw wrapped;
    } catch (simErr) {
      // Fall back to original error if simulation also fails.
      throw err instanceof Error ? err : new Error(String(err));
    }
  }
}

export class SolanaService {
  private connection: Connection;
  private programId: PublicKey;

  constructor() {
    this.connection = new Connection(RPC_URL, 'confirmed');
    if (!VAULT_PROGRAM_ID) {
      throw new Error('Missing VITE_VAULT_PROGRAM_ID');
    }
    this.programId = new PublicKey(VAULT_PROGRAM_ID);
  }

  getProgramId(): string {
    return this.programId.toBase58();
  }

  getCluster(): string {
    return SOLANA_NETWORK;
  }

  getConnection(): Connection {
    return this.connection;
  }

  getRpcUrl(): string {
    return RPC_URL;
  }

  async getBalance(publicKey: string, connection?: Connection): Promise<number> {
    const conn = connection ?? this.connection;
    const pubKey = new PublicKey(publicKey);
    const balance = await withRateLimitRetry(() => conn.getBalance(pubKey));
    return balance / 1_000_000_000; // Convert to SOL
  }

  async deriveVaultPDA(userPublicKey: string): Promise<string> {
    try {
      const userPubKey = new PublicKey(userPublicKey);
      const casino = await this.deriveCasinoPDA();
      const [vaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('vault'), new PublicKey(casino).toBuffer(), userPubKey.toBuffer()],
        this.programId
      );
      return vaultPDA.toBase58();
    } catch (error) {
      console.error('Failed to derive vault PDA:', error);
      throw error;
    }
  }

  async deriveCasinoPDA(): Promise<string> {
    const [casinoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('casino')],
      this.programId
    );
    return casinoPda.toBase58();
  }

  async deriveVaultAuthorityPDA(): Promise<string> {
    const casino = new PublicKey(await this.deriveCasinoPDA());
    const [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault-authority'), casino.toBuffer()],
      this.programId
    );
    return vaultAuthority.toBase58();
  }

  async deriveRateLimiterPDA(userPublicKey: string): Promise<string> {
    const user = new PublicKey(userPublicKey);
    const [rateLimiter] = PublicKey.findProgramAddressSync(
      [Buffer.from('rate-limiter'), user.toBuffer()],
      this.programId
    );
    return rateLimiter.toBase58();
  }

  async deriveAllowanceNonceRegistryPDA(params: { userPublicKey: string; casinoPda?: string }): Promise<string> {
    const user = new PublicKey(params.userPublicKey);
    const casino = new PublicKey(params.casinoPda ?? (await this.deriveCasinoPDA()));
    const [registry] = PublicKey.findProgramAddressSync(
      [Buffer.from('allowance-nonce'), user.toBuffer(), casino.toBuffer()],
      this.programId
    );
    return registry.toBase58();
  }

  async getNextAllowanceNonce(params: {
    user: PublicKey;
    casinoPda?: string;
    connection?: Connection;
  }): Promise<bigint> {
    const conn = params.connection ?? this.connection;
    const casinoPda = params.casinoPda ?? (await this.deriveCasinoPDA());
    const registryPda = await this.deriveAllowanceNonceRegistryPDA({
      userPublicKey: params.user.toBase58(),
      casinoPda,
    });

    const info = await withRateLimitRetry(() => conn.getAccountInfo(new PublicKey(registryPda), 'confirmed'));
    if (!info) return 0n;
    const data = Buffer.from(info.data);
    const state = parseAllowanceNonceRegistryAccount(data);
    return state.nextNonce;
  }

  async getAccountExists(address: string, connection?: Connection): Promise<boolean> {
    const conn = connection ?? this.connection;
    const info = await withRateLimitRetry(() => conn.getAccountInfo(new PublicKey(address), 'confirmed'));
    return info !== null;
  }

  async getVaultInfoByAddress(
    address: string,
    connection?: Connection
  ): Promise<{
    exists: boolean;
    address: string;
    lamports: number | null;
    state: VaultAccountState | null;
  }> {
    const conn = connection ?? this.connection;
    const info = await withRateLimitRetry(() => conn.getAccountInfo(new PublicKey(address), 'confirmed'));
    if (!info) {
      return { exists: false, address, lamports: null, state: null };
    }

    const data = Buffer.from(info.data);
    const state = parseVaultAccount(data);
    return {
      exists: true,
      address,
      lamports: info.lamports,
      state,
    };
  }

  async getUserVaultInfo(params: {
    user: PublicKey;
    connection?: Connection;
  }): Promise<{
    exists: boolean;
    address: string;
    lamports: number | null;
    state: VaultAccountState | null;
  }> {
    const vaultPda = await this.deriveVaultPDA(params.user.toBase58());
    return this.getVaultInfoByAddress(vaultPda, params.connection);
  }

  async getAllowanceInfoByAddress(
    address: string,
    connection?: Connection
  ): Promise<{
    exists: boolean;
    address: string;
    lamports: number | null;
    state: AllowanceAccountState | null;
  }> {
    const conn = connection ?? this.connection;
    const info = await withRateLimitRetry(() => conn.getAccountInfo(new PublicKey(address), 'confirmed'));
    if (!info) {
      return { exists: false, address, lamports: null, state: null };
    }

    const data = Buffer.from(info.data);
    const state = parseAllowanceAccount(data);
    return {
      exists: true,
      address,
      lamports: info.lamports,
      state,
    };
  }

  async initializeCasinoVault(params: {
    authority: PublicKey;
    sendTransaction: SendTransactionFn;
    connection?: Connection;
  }): Promise<{ signature: string; casinoPda: string; vaultAuthorityPda: string }> {
    const connection = params.connection ?? this.connection;
    const casinoPda = await this.deriveCasinoPDA();
    const vaultAuthorityPda = await this.deriveVaultAuthorityPDA();

    const data = await buildIxData('initialize_casino_vault', [params.authority.toBuffer()]);

    const ix = new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: new PublicKey(casinoPda), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(vaultAuthorityPda), isSigner: false, isWritable: false },
        { pubkey: params.authority, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });

    const tx = new Transaction().add(ix);
    tx.feePayer = params.authority;

    const signature = await sendAndConfirm(connection, params.sendTransaction, tx);
    return { signature, casinoPda, vaultAuthorityPda };
  }

  async initializeUserVault(params: {
    user: PublicKey;
    sendTransaction: SendTransactionFn;
    connection?: Connection;
  }): Promise<{ signature: string; vaultPda: string; casinoPda: string }> {
    const connection = params.connection ?? this.connection;
    const casinoPda = await this.deriveCasinoPDA();
    const vaultPda = await this.deriveVaultPDA(params.user.toBase58());

    const data = await buildIxData('initialize_vault');

    const ix = new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: new PublicKey(vaultPda), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(casinoPda), isSigner: false, isWritable: false },
        { pubkey: params.user, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });

    const tx = new Transaction().add(ix);
    tx.feePayer = params.user;

    const signature = await sendAndConfirm(connection, params.sendTransaction, tx);
    return { signature, vaultPda, casinoPda };
  }

  async depositSol(params: {
    user: PublicKey;
    amountLamports: bigint;
    sendTransaction: SendTransactionFn;
    connection?: Connection;
  }): Promise<{ signature: string; vaultPda: string }> {
    const connection = params.connection ?? this.connection;
    const casinoPda = await this.deriveCasinoPDA();
    const vaultPda = await this.deriveVaultPDA(params.user.toBase58());

    const data = await buildIxData('deposit_sol', [u64ToLeBytes(params.amountLamports)]);

    const ix = new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: new PublicKey(vaultPda), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(casinoPda), isSigner: false, isWritable: false },
        { pubkey: params.user, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });

    const tx = new Transaction().add(ix);
    tx.feePayer = params.user;
    const signature = await sendAndConfirm(connection, params.sendTransaction, tx);
    return { signature, vaultPda };
  }

  async withdrawSol(params: {
    user: PublicKey;
    amountLamports: bigint;
    sendTransaction: SendTransactionFn;
    connection?: Connection;
  }): Promise<{ signature: string; vaultPda: string }> {
    const connection = params.connection ?? this.connection;
    const casinoPda = await this.deriveCasinoPDA();
    const vaultPda = await this.deriveVaultPDA(params.user.toBase58());

    const data = await buildIxData('withdraw_sol', [u64ToLeBytes(params.amountLamports)]);

    const ix = new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: new PublicKey(vaultPda), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(casinoPda), isSigner: false, isWritable: false },
        { pubkey: params.user, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });

    const tx = new Transaction().add(ix);
    tx.feePayer = params.user;
    const signature = await sendAndConfirm(connection, params.sendTransaction, tx);
    return { signature, vaultPda };
  }

  async approveAllowanceSol(params: {
    user: PublicKey;
    amountLamports: bigint;
    durationSeconds: bigint;
    sendTransaction: SendTransactionFn;
    signTransaction?: SignTransactionFn;
    signAllTransactions?: SignAllTransactionsFn;
    connection?: Connection;
  }): Promise<{ signature: string; allowancePda: string; usedNonce: bigint }> {
    const connection = params.connection ?? this.connection;

    const casinoPda = await this.deriveCasinoPDA();
    const vaultPda = await this.deriveVaultPDA(params.user.toBase58());
    const rateLimiterPda = await this.deriveRateLimiterPDA(params.user.toBase58());

    // Nonce-based deterministic allowance PDA (v2):
    // - Read `next_nonce` from AllowanceNonceRegistry PDA (or 0 if missing)
    // - Derive Allowance PDA using that nonce
    // - Call `approve_allowance_v2` with the nonce

    const casinoPk = new PublicKey(casinoPda);
    const nonce = await this.getNextAllowanceNonce({ user: params.user, casinoPda, connection });
    const [registryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('allowance-nonce'), params.user.toBuffer(), casinoPk.toBuffer()],
      this.programId
    );
    const [allowancePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('allowance'), params.user.toBuffer(), casinoPk.toBuffer(), u64ToLeBytes(nonce)],
      this.programId
    );

    const data = await buildIxData('approve_allowance_v2', [
      u64ToLeBytes(params.amountLamports),
      i64ToLeBytes(params.durationSeconds),
      SystemProgram.programId.toBuffer(),
      u64ToLeBytes(nonce),
    ]);

    const ix = new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: new PublicKey(vaultPda), isSigner: false, isWritable: true },
        { pubkey: casinoPk, isSigner: false, isWritable: false },
        { pubkey: registryPda, isSigner: false, isWritable: true },
        { pubkey: allowancePda, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(rateLimiterPda), isSigner: false, isWritable: true },
        { pubkey: params.user, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });

    const tx = new Transaction().add(ix);
    tx.feePayer = params.user;
    try {
      const signature = params.signTransaction
        ? await signSendAndConfirm(connection, params.signTransaction, tx)
        : await sendAndConfirm(connection, params.sendTransaction, tx);
      return { signature, allowancePda: allowancePda.toBase58(), usedNonce: nonce };
    } catch (err) {
      if (isUserRejectedError(err)) throw new Error('User cancelled allowance approval');
      throw err;
    }
  }

  async revokeAllowance(params: {
    user: PublicKey;
    allowancePda: PublicKey;
    sendTransaction: SendTransactionFn;
    connection?: Connection;
  }): Promise<{ signature: string }> {
    const connection = params.connection ?? this.connection;
    const data = await buildIxData('revoke_allowance');

    const ix = new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: params.allowancePda, isSigner: false, isWritable: true },
        { pubkey: params.user, isSigner: true, isWritable: false },
      ],
      data,
    });

    const tx = new Transaction().add(ix);
    tx.feePayer = params.user;
    const signature = await sendAndConfirm(connection, params.sendTransaction, tx);
    return { signature };
  }

  async requestAirdrop(publicKey: string, amount: number = 1, connection?: Connection): Promise<string> {
    const conn = connection ?? this.connection;
    const pubKey = new PublicKey(publicKey);
    const signature = await withRateLimitRetry(() =>
      conn.requestAirdrop(
        pubKey,
        amount * 1_000_000_000 // Convert SOL to lamports
      )
    );

    // Wait for confirmation
    await withRateLimitRetry(() => conn.confirmTransaction(signature));
    return signature;
  }

  getExplorerUrl(signature: string, cluster?: string): string {
    const c = cluster ?? SOLANA_NETWORK;
    return `https://explorer.solana.com/tx/${signature}?cluster=${c}`;
  }

  getAccountExplorerUrl(address: string, cluster?: string): string {
    const c = cluster ?? SOLANA_NETWORK;
    return `https://explorer.solana.com/address/${address}?cluster=${c}`;
  }
}

export const solanaService = new SolanaService();