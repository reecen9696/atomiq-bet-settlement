import { PublicKey, TransactionInstruction } from '@solana/web3.js';

/**
 * Memo program ID for adding human-readable messages to transactions
 */
export const MEMO_PROGRAM_ID = new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr');

/**
 * Create a memo instruction with a human-readable message
 * This will be displayed in wallet popups like Phantom
 */
export function createMemoInstruction(message: string): TransactionInstruction {
  return new TransactionInstruction({
    keys: [],
    programId: MEMO_PROGRAM_ID,
    data: Buffer.from(message, 'utf8'),
  });
}

/**
 * Generate memo messages for different transaction types
 */
export const MemoMessages = {
  initializeVault: () => 'Initialize your Atomik vault for secure betting',
  
  depositSol: (amount: number) => `Deposit ${amount} SOL to your Atomik vault`,
  
  withdrawSol: (amount: number) => `Withdraw ${amount} SOL from your Atomik vault`,
  
  airdrop: (amount: number) => `Request ${amount} SOL airdrop (devnet only)`,
  
  approveAllowance: (amount: number, expiryDate?: string) => {
    const expiry = expiryDate ? ` until ${expiryDate}` : '';
    return `Approve session key to spend up to ${amount} SOL for bets${expiry}`;
  },
  
  revokeAllowance: () => 'Revoke session key spending permission',
  
  placeBet: (choice: 'heads' | 'tails', amount: number) => 
    `Bet ${amount} SOL on ${choice} - Coinflip Game`,
  
  settleGame: (gameId: string) => `Settle game ${gameId} and claim winnings`,
  
  createCasino: () => 'Initialize Atomik casino (admin only)',
  
  updateCasinoSettings: () => 'Update casino settings (admin only)',
};

/**
 * Truncate memo message if it's too long (wallets may truncate anyway)
 */
export function truncateMemo(message: string, maxLength: number = 100): string {
  if (message.length <= maxLength) return message;
  return message.substring(0, maxLength - 3) + '...';
}