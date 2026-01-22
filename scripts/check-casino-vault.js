#!/usr/bin/env node

/**
 * Check and fund the casino vault PDA
 * This vault holds the SOL used to pay out winning bets
 */

const {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  Keypair,
  LAMPORTS_PER_SOL,
} = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

// Configuration
const RPC_URL = process.env.SOLANA_RPC_URL || "https://api.devnet.solana.com";
const VAULT_PROGRAM_ID = new PublicKey(
  process.env.VAULT_PROGRAM_ID ||
    "BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ",
);

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");

  // Derive Casino PDA
  const [casinoPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino")],
    VAULT_PROGRAM_ID,
  );

  // Derive Casino Vault PDA (this is what holds the SOL)
  const [casinoVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino-vault"), casinoPda.toBuffer()],
    VAULT_PROGRAM_ID,
  );

  // Derive Vault Authority PDA
  const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault-authority"), casinoPda.toBuffer()],
    VAULT_PROGRAM_ID,
  );

  console.log("=== Casino Vault Check ===\n");
  console.log("Casino PDA:", casinoPda.toBase58());
  console.log("Casino Vault PDA:", casinoVaultPda.toBase58());
  console.log("Vault Authority PDA:", vaultAuthorityPda.toBase58());
  console.log();

  // Check Casino PDA balance
  const casinoPdaBalance = await connection.getBalance(casinoPda);
  console.log(`Casino PDA Balance: ${casinoPdaBalance / LAMPORTS_PER_SOL} SOL`);
  console.log("  (This is the state account, not the vault)");

  // Check Casino Vault balance
  const vaultBalance = await connection.getBalance(casinoVaultPda);
  console.log(`\nCasino Vault Balance: ${vaultBalance / LAMPORTS_PER_SOL} SOL`);
  console.log("  (This is the account that pays out winnings)");

  // Check if vault exists
  const vaultAccount = await connection.getAccountInfo(casinoVaultPda);
  if (!vaultAccount) {
    console.log("\n⚠️  Casino Vault account does not exist!");
    console.log("   The vault account needs to be created first.");
    return;
  }

  // Calculate recommended balance
  const recommendedBalance = 10 * LAMPORTS_PER_SOL; // 10 SOL
  const deficit = recommendedBalance - vaultBalance;

  console.log(
    `\nRecommended Balance: ${recommendedBalance / LAMPORTS_PER_SOL} SOL`,
  );

  if (deficit > 0) {
    console.log(`\n❌ INSUFFICIENT FUNDS!`);
    console.log(`   Need to add: ${deficit / LAMPORTS_PER_SOL} SOL`);
    console.log(`\n💡 To fund the vault, run:`);
    console.log(
      `   node scripts/fund-casino-vault.js ${deficit / LAMPORTS_PER_SOL}`,
    );
  } else {
    console.log(`\n✅ Casino vault is sufficiently funded!`);
  }

  // Calculate how many max bets can be paid out
  const maxBetPayout = 0.02 * LAMPORTS_PER_SOL; // 0.02 SOL per winning bet
  const maxWinningBets = Math.floor(vaultBalance / maxBetPayout);
  console.log(`\nCan pay out ${maxWinningBets} winning bets (0.02 SOL each)`);
}

main().catch((err) => {
  console.error("Error:", err);
  process.exit(1);
});
