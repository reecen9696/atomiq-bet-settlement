#!/usr/bin/env node
// Fund the casino vault authority PDA

const {
  PublicKey,
  Connection,
  Keypair,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
} = require("@solana/web3.js");
const fs = require("fs");

const PROGRAM_ID = new PublicKey(
  "BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ",
);
const RPC_URL = "https://api.devnet.solana.com";

async function main() {
  const amount = process.argv[2] || "1.0";
  const keypairPath =
    process.argv[3] || `${process.env.HOME}/.config/solana/id.json`;

  console.log("🏦 Fund Casino Vault");
  console.log("===================\n");

  // Load funder keypair
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf8"));
  const funder = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  // Derive PDAs
  const [casinoPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino")],
    PROGRAM_ID,
  );

  const [vaultAuthorityPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault-authority"), casinoPDA.toBuffer()],
    PROGRAM_ID,
  );

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("Casino PDA:", casinoPDA.toBase58());
  console.log("Vault Authority PDA:", vaultAuthorityPDA.toBase58());
  console.log("Funder:", funder.publicKey.toBase58());
  console.log("Amount:", amount, "SOL\n");

  // Connect and check balances
  const connection = new Connection(RPC_URL, "confirmed");

  const funderBalance = await connection.getBalance(funder.publicKey);
  const vaultBalance = await connection.getBalance(vaultAuthorityPDA);

  console.log("Current Balances:");
  console.log(
    "  Funder:",
    (funderBalance / LAMPORTS_PER_SOL).toFixed(4),
    "SOL",
  );
  console.log(
    "  Vault Authority:",
    (vaultBalance / LAMPORTS_PER_SOL).toFixed(4),
    "SOL\n",
  );

  // Create transfer transaction
  const lamports = Math.floor(parseFloat(amount) * LAMPORTS_PER_SOL);

  console.log("Transferring", lamports, "lamports...");

  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: funder.publicKey,
      toPubkey: vaultAuthorityPDA,
      lamports,
    }),
  );

  const signature = await connection.sendTransaction(tx, [funder]);
  await connection.confirmTransaction(signature, "confirmed");

  console.log("✅ Transfer complete!");
  console.log("Signature:", signature);
  console.log(`https://explorer.solana.com/tx/${signature}?cluster=devnet\n`);

  // Check new balance
  const newVaultBalance = await connection.getBalance(vaultAuthorityPDA);
  console.log(
    "New Vault Authority Balance:",
    (newVaultBalance / LAMPORTS_PER_SOL).toFixed(4),
    "SOL",
  );
}

main().catch((err) => {
  console.error("Error:", err.message);
  process.exit(1);
});
