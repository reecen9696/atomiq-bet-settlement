/**
 * Example implementation showing how to add memo instructions to Solana transactions
 * This demonstrates the pattern for all transaction types in the SDK
 */

import {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
} from "@solana/web3.js";
import { createMemoInstruction, MemoMessages } from "../utils/memo";

/**
 * Example: Building a deposit transaction with memo instruction
 * This is how the actual implementation would work when integrated
 */
export async function buildDepositTransactionWithMemo(params: {
  userPublicKey: PublicKey;
  vaultPda: PublicKey;
  amount: number; // in SOL
  connection: Connection;
}): Promise<Transaction> {
  const { userPublicKey, vaultPda, amount, connection } = params;

  // Create a new transaction
  const transaction = new Transaction();

  // 1. Add memo instruction FIRST (so it's prominent in wallet UI)
  const memoInstruction = createMemoInstruction(
    MemoMessages.depositSol(amount),
  );
  transaction.add(memoInstruction);

  // 2. Add the actual program instruction(s) after memo
  const depositInstruction = SystemProgram.transfer({
    fromPubkey: userPublicKey,
    toPubkey: vaultPda,
    lamports: amount * 1e9, // Convert SOL to lamports
  });
  transaction.add(depositInstruction);

  // 3. Set recent blockhash and fee payer
  const { blockhash } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = userPublicKey;

  return transaction;
}

/**
 * Example: Building a betting transaction with memo instruction
 */
export async function buildBettingTransactionWithMemo(params: {
  userPublicKey: PublicKey;
  choice: "heads" | "tails";
  amount: number;
  connection: Connection;
}): Promise<Transaction> {
  const { userPublicKey, choice, amount, connection } = params;

  const transaction = new Transaction();

  // 1. Add memo instruction FIRST
  const memoInstruction = createMemoInstruction(
    MemoMessages.placeBet(choice, amount),
  );
  transaction.add(memoInstruction);

  // 2. Add betting program instruction
  // (This would be your actual program instruction for placing bets)
  // const bettingInstruction = new TransactionInstruction({
  //   keys: [...], // Account metas
  //   programId,
  //   data: Buffer.from([...]) // Instruction data
  // });
  // transaction.add(bettingInstruction);

  // Set transaction properties
  const { blockhash } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = userPublicKey;

  return transaction;
}

/**
 * Example: Building an allowance approval transaction
 */
export async function buildApproveAllowanceTransactionWithMemo(params: {
  userPublicKey: PublicKey;
  amount: number;
  expiryDate?: string;
  connection: Connection;
}): Promise<Transaction> {
  const { userPublicKey, amount, expiryDate, connection } = params;

  const transaction = new Transaction();

  // 1. Add memo instruction FIRST
  const memoInstruction = createMemoInstruction(
    MemoMessages.approveAllowance(amount, expiryDate),
  );
  transaction.add(memoInstruction);

  // 2. Add allowance approval instruction
  // (This would be your actual program instruction for approving allowances)
  // const approveInstruction = new TransactionInstruction({
  //   keys: [...],
  //   programId,
  //   data: Buffer.from([...])
  // });
  // transaction.add(approveInstruction);

  const { blockhash } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = userPublicKey;

  return transaction;
}

/**
 * Integration example: How to use with wallet-adapter
 */
export async function executeTransactionWithWallet(params: {
  transaction: Transaction;
  connection: Connection;
  sendTransaction: (
    transaction: Transaction,
    connection: Connection,
  ) => Promise<string>;
}) {
  const { transaction, connection, sendTransaction } = params;

  try {
    // Send transaction through wallet
    // The memo instruction will appear in the wallet popup
    const signature = await sendTransaction(transaction, connection);

    // Wait for confirmation
    await connection.confirmTransaction(signature);

    return signature;
  } catch (error) {
    console.error("Transaction failed:", error);
    throw error;
  }
}

/**
 * React hook example: How to use in components
 */
export function useTransactionWithMemo() {
  return {
    buildDepositWithMemo: buildDepositTransactionWithMemo,
    buildBetWithMemo: buildBettingTransactionWithMemo,
    buildApproveWithMemo: buildApproveAllowanceTransactionWithMemo,
    executeWithWallet: executeTransactionWithWallet,
  };
}
