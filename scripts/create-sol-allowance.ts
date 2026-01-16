import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import fs from "fs";

const PROGRAM_ID = new PublicKey("HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4");

async function main() {
  // Setup provider
  const connection = new anchor.web3.Connection("https://api.devnet.solana.com", "confirmed");
  
  // Load user keypair
  const userKeypairData = JSON.parse(fs.readFileSync("/Users/reece/code/projects/atomik-wallet/test-user-keypair.json", "utf-8"));
  const userKeypair = Keypair.fromSecretKey(Uint8Array.from(userKeypairData));
  
  const wallet = new anchor.Wallet(userKeypair);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  
  console.log("User:", userKeypair.publicKey.toString());
  
  // Derive PDAs
  const [casino] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino")],
    PROGRAM_ID
  );
  
  const [userVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), casino.toBuffer(), userKeypair.publicKey.toBuffer()],
    PROGRAM_ID
  );
  
  const [rateLimiter] = PublicKey.findProgramAddressSync(
    [Buffer.from("rate-limiter"), userKeypair.publicKey.toBuffer()],
    PROGRAM_ID
  );
  
  console.log("Casino:", casino.toString());
  console.log("User Vault:", userVault.toString());
  console.log("Rate Limiter:", rateLimiter.toString());
  
  // Get current time to derive allowance PDA
  const slot = await connection.getSlot();
  const blockTime = await connection.getBlockTime(slot);
  const timestamp = new anchor.BN(blockTime!);
  
  const [allowance] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("allowance"),
      userKeypair.publicKey.toBuffer(),
      casino.toBuffer(),
      timestamp.toArrayLike(Buffer, "le", 8),
    ],
    PROGRAM_ID
  );
  
  console.log("Allowance PDA:", allowance.toString());
  console.log("Timestamp:", blockTime);
  
  // Build approve_allowance instruction manually
  const amount = new anchor.BN(1_000_000_000); // 1 SOL
  const duration = new anchor.BN(86400); // 24 hours
  const tokenMint = SystemProgram.programId; // System::id() for native SOL
  
  // Discriminator for approve_allowance
  const discriminator = Buffer.from([100, 169, 165, 25, 25, 255, 11, 45]);
  
  const data = Buffer.concat([
    discriminator,
    amount.toArrayLike(Buffer, "le", 8),
    duration.toArrayLike(Buffer, "le", 8),
    tokenMint.toBuffer(),
  ]);
  
  const instruction = new anchor.web3.TransactionInstruction({
    keys: [
      { pubkey: userVault, isSigner: false, isWritable: true },
      { pubkey: casino, isSigner: false, isWritable: false },
      { pubkey: allowance, isSigner: false, isWritable: true },
      { pubkey: rateLimiter, isSigner: false, isWritable: true },
      { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });
  
  console.log("\nApproving 1 SOL allowance for 24 hours...");
  
  const tx = new anchor.web3.Transaction().add(instruction);
  const sig = await provider.sendAndConfirm(tx);
  
  console.log("✅ Allowance Created!");
  console.log("Signature:", sig);
  console.log("\nAllowance PDA:", allowance.toString());
  console.log("\nUpdate processor to use this allowance address.");
}

main().catch(console.error);
