#!/usr/bin/env node

/**
 * CLI script to approve an allowance directly
 * Usage: node scripts/approve-allowance-cli.js <keypair-path> <amount-sol> <duration-seconds>
 */

const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require("@solana/web3.js");
const fs = require("fs");
const crypto = require("crypto");

const RPC_URL =
  process.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
const PROGRAM_ID = new PublicKey(
  process.env.VITE_VAULT_PROGRAM_ID ||
    "BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ",
);
const MEMO_PROGRAM_ID = new PublicKey(
  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
);

function u64ToLeBytes(value) {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(BigInt(value));
  return buf;
}

function i64ToLeBytes(value) {
  const buf = Buffer.alloc(8);
  buf.writeBigInt64LE(BigInt(value));
  return buf;
}

async function anchorDiscriminator(ixName) {
  const preimage = Buffer.from(`global:${ixName}`, "utf-8");
  const hash = crypto.createHash("sha256").update(preimage).digest();
  return hash.subarray(0, 8);
}

async function buildIxData(ixName, args = []) {
  const disc = await anchorDiscriminator(ixName);
  return Buffer.concat([disc, ...args]);
}

function createUniqueMemoInstruction() {
  const memo = `atomik-cli-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  const memoData = Buffer.from(memo, "utf-8");

  return new TransactionInstruction({
    keys: [],
    programId: MEMO_PROGRAM_ID,
    data: memoData,
  });
}

async function getNextAllowanceNonce(connection, user, casino) {
  const [registryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("allowance-nonce"), user.toBuffer(), casino.toBuffer()],
    PROGRAM_ID,
  );

  try {
    const info = await connection.getAccountInfo(registryPda, "confirmed");
    if (!info) return 0n;

    const data = info.data;
    // Skip discriminator (8 bytes) + user (32) + casino (32) = 72 bytes
    const nonce = data.readBigUInt64LE(72);
    return nonce;
  } catch (err) {
    console.log("Registry not found, using nonce 0");
    return 0n;
  }
}

async function main() {
  const args = process.argv.slice(2);

  if (args.length < 3) {
    console.error(
      "Usage: node scripts/approve-allowance-cli.js <keypair-path> <amount-sol> <duration-seconds>",
    );
    console.error(
      "Example: node scripts/approve-allowance-cli.js ~/.config/solana/id.json 0.1 3600",
    );
    process.exit(1);
  }

  const [keypairPath, amountSol, durationSeconds] = args;

  // Load keypair
  const keypairContent = fs.readFileSync(keypairPath, "utf-8").trim();
  let keypair;

  try {
    // Try parsing as JSON array first
    const keypairData = JSON.parse(keypairContent);
    keypair = Keypair.fromSecretKey(Uint8Array.from(keypairData));
  } catch (err) {
    // Try as base58 string (Phantom export format)
    const bs58 = require("bs58");
    keypair = Keypair.fromSecretKey(bs58.decode(keypairContent));
  }

  console.log("👤 User:", keypair.publicKey.toBase58());

  // Connect
  const connection = new Connection(RPC_URL, "confirmed");

  // Derive PDAs
  const [casinoPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino")],
    PROGRAM_ID,
  );

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), casinoPda.toBuffer(), keypair.publicKey.toBuffer()],
    PROGRAM_ID,
  );

  const [rateLimiterPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("rate-limiter"), keypair.publicKey.toBuffer()],
    PROGRAM_ID,
  );

  // Get nonce
  const nonce = await getNextAllowanceNonce(
    connection,
    keypair.publicKey,
    casinoPda,
  );
  console.log("🔢 Nonce:", nonce.toString());

  const [registryPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("allowance-nonce"),
      keypair.publicKey.toBuffer(),
      casinoPda.toBuffer(),
    ],
    PROGRAM_ID,
  );

  const [allowancePda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("allowance"),
      keypair.publicKey.toBuffer(),
      casinoPda.toBuffer(),
      u64ToLeBytes(nonce),
    ],
    PROGRAM_ID,
  );

  console.log("🏦 Casino PDA:", casinoPda.toBase58());
  console.log("🔐 Vault PDA:", vaultPda.toBase58());
  console.log("✅ Allowance PDA:", allowancePda.toBase58());
  console.log("💰 Amount:", amountSol, "SOL");
  console.log("⏱️  Duration:", durationSeconds, "seconds");

  // Build instruction data
  const amountLamports = BigInt(Math.floor(parseFloat(amountSol) * 1e9));
  const duration = BigInt(durationSeconds);

  const data = await buildIxData("approve_allowance_v2", [
    u64ToLeBytes(amountLamports),
    i64ToLeBytes(duration),
    SystemProgram.programId.toBuffer(),
    u64ToLeBytes(nonce),
  ]);

  // Build instruction
  const ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: vaultPda, isSigner: false, isWritable: true },
      { pubkey: casinoPda, isSigner: false, isWritable: false },
      { pubkey: registryPda, isSigner: false, isWritable: true },
      { pubkey: allowancePda, isSigner: false, isWritable: true },
      { pubkey: rateLimiterPda, isSigner: false, isWritable: true },
      { pubkey: keypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });

  // Create unique memo
  const memoIx = createUniqueMemoInstruction();
  console.log("📝 Memo:", memoIx.data.toString("utf-8"));

  // Build and send transaction
  const tx = new Transaction().add(memoIx).add(ix);
  tx.feePayer = keypair.publicKey;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  console.log("\n🚀 Sending transaction...");

  try {
    const signature = await sendAndConfirmTransaction(
      connection,
      tx,
      [keypair],
      {
        skipPreflight: false,
        commitment: "confirmed",
      },
    );

    console.log("✅ Success!");
    console.log("📜 Signature:", signature);
    console.log(
      "🔗 Explorer:",
      `https://explorer.solana.com/tx/${signature}?cluster=devnet`,
    );
    console.log("✅ Allowance PDA:", allowancePda.toBase58());
  } catch (err) {
    console.error("❌ Transaction failed:", err.message);
    if (err.logs) {
      console.error("Logs:", err.logs);
    }
    process.exit(1);
  }
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
