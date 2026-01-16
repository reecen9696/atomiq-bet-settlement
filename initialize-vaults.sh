#!/bin/bash

# Initialize Vaults Script
# This must be run ONCE before testing bets on-chain

set -e

echo "🔧 Initializing Vaults for On-Chain Testing"
echo "=============================================="
echo ""

# Load wallet addresses
PROCESSOR_WALLET=$(solana-keygen pubkey test-keypair.json)
USER_WALLET=$(solana-keygen pubkey test-user-keypair.json)
PROGRAM_ID="HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4"

echo "📝 Program ID: $PROGRAM_ID"
echo "🔑 Processor Wallet: $PROCESSOR_WALLET"
echo "👤 User Wallet: $USER_WALLET"
echo ""

# Check balances
echo "💰 Current balances:"
PROCESSOR_BAL=$(solana balance $PROCESSOR_WALLET --url devnet | awk '{print $1}')
USER_BAL=$(solana balance $USER_WALLET --url devnet | awk '{print $1}')
echo "  Processor: $PROCESSOR_BAL SOL"
echo "  User: $USER_BAL SOL"
echo ""

# We'll use the Anchor CLI to call the program
# First, let's create a simple TypeScript file to do the initialization

cat > /tmp/init-vaults.ts << 'EOF'
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as fs from "fs";

// Load keypairs
const processorKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("test-keypair.json", "utf-8")))
);
const userKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("test-user-keypair.json", "utf-8")))
);

const programId = new PublicKey("HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4");
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

async function main() {
  console.log("🔧 Starting vault initialization...\n");

  // 1. Initialize User Vault
  console.log("1️⃣ Initializing user vault...");
  const [userVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("user_vault"), userKeypair.publicKey.toBuffer()],
    programId
  );
  console.log(`   User Vault PDA: ${userVault.toString()}`);

  try {
    // Build initialize_vault instruction manually
    const initVaultIx = {
      keys: [
        { pubkey: userVault, isSigner: false, isWritable: true },
        { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: programId,
      data: Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]), // initialize_vault discriminator
    };

    const tx = new anchor.web3.Transaction().add(initVaultIx);
    const sig = await connection.sendTransaction(tx, [userKeypair]);
    await connection.confirmTransaction(sig, "confirmed");
    console.log(`   ✅ User vault initialized: ${sig}`);
  } catch (e: any) {
    if (e.message.includes("already in use")) {
      console.log(`   ℹ️  User vault already initialized`);
    } else {
      console.log(`   ❌ Error: ${e.message}`);
    }
  }

  // 2. Deposit SOL into user vault
  console.log("\n2️⃣ Depositing SOL into user vault...");
  const depositAmount = 500_000_000; // 0.5 SOL
  try {
    const depositIx = {
      keys: [
        { pubkey: userVault, isSigner: false, isWritable: true },
        { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: programId,
      data: Buffer.concat([
        Buffer.from([242, 35, 198, 137, 82, 225, 242, 182]), // deposit_sol discriminator
        Buffer.from(new anchor.BN(depositAmount).toArray("le", 8)),
      ]),
    };

    const tx = new anchor.web3.Transaction().add(depositIx);
    const sig = await connection.sendTransaction(tx, [userKeypair]);
    await connection.confirmTransaction(sig, "confirmed");
    console.log(`   ✅ Deposited ${depositAmount / 1e9} SOL: ${sig}`);
  } catch (e: any) {
    console.log(`   ❌ Error: ${e.message}`);
  }

  // 3. Initialize Casino Vault
  console.log("\n3️⃣ Initializing casino vault...");
  const [casinoVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino_vault")],
    programId
  );
  console.log(`   Casino Vault PDA: ${casinoVault.toString()}`);

  try {
    const initCasinoIx = {
      keys: [
        { pubkey: casinoVault, isSigner: false, isWritable: true },
        { pubkey: processorKeypair.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: programId,
      data: Buffer.concat([
        Buffer.from([229, 50, 107, 196, 255, 176, 205, 206]), // initialize_casino_vault discriminator
        processorKeypair.publicKey.toBuffer(),
      ]),
    };

    const tx = new anchor.web3.Transaction().add(initCasinoIx);
    const sig = await connection.sendTransaction(tx, [processorKeypair]);
    await connection.confirmTransaction(sig, "confirmed");
    console.log(`   ✅ Casino vault initialized: ${sig}`);
  } catch (e: any) {
    if (e.message.includes("already in use")) {
      console.log(`   ℹ️  Casino vault already initialized`);
    } else {
      console.log(`   ❌ Error: ${e.message}`);
    }
  }

  // 4. Approve Allowance
  console.log("\n4️⃣ Approving spending allowance...");
  const allowanceAmount = 1_000_000_000; // 1 SOL
  const durationSeconds = 86400; // 24 hours
  const nativeMint = new PublicKey("So11111111111111111111111111111111111111112");

  const [allowance] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("allowance"),
      userKeypair.publicKey.toBuffer(),
      processorKeypair.publicKey.toBuffer(),
    ],
    programId
  );

  try {
    const approveIx = {
      keys: [
        { pubkey: allowance, isSigner: false, isWritable: true },
        { pubkey: userVault, isSigner: false, isWritable: false },
        { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
        { pubkey: processorKeypair.publicKey, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: programId,
      data: Buffer.concat([
        Buffer.from([222, 213, 208, 207, 253, 39, 94, 112]), // approve_allowance discriminator
        Buffer.from(new anchor.BN(allowanceAmount).toArray("le", 8)),
        Buffer.from(new anchor.BN(durationSeconds).toArray("le", 8)),
        nativeMint.toBuffer(),
      ]),
    };

    const tx = new anchor.web3.Transaction().add(approveIx);
    const sig = await connection.sendTransaction(tx, [userKeypair]);
    await connection.confirmTransaction(sig, "confirmed");
    console.log(`   ✅ Allowance approved for ${allowanceAmount / 1e9} SOL: ${sig}`);
  } catch (e: any) {
    if (e.message.includes("already in use")) {
      console.log(`   ℹ️  Allowance already approved`);
    } else {
      console.log(`   ❌ Error: ${e.message}`);
    }
  }

  console.log("\n✅ Vault initialization complete!");
  console.log("\nYou can now run: ./test-onchain.sh");
}

main().catch((e) => {
  console.error("Fatal error:", e);
  process.exit(1);
});
EOF

echo "⏳ Running initialization (this may take a minute)..."
echo ""

# Run with tsx (TypeScript executor)
if ! command -v tsx &> /dev/null; then
    echo "Installing tsx..."
    npm install -g tsx @coral-xyz/anchor @solana/web3.js
fi

cd /Users/reece/code/projects/atomik-wallet
tsx /tmp/init-vaults.ts

echo ""
echo "✅ Done! You can now run ./test-onchain.sh"
