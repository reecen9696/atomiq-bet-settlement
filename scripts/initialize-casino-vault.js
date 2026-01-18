/**
 * Initialize CasinoVault and migrate funds from old vault_authority PDA
 *
 * This script:
 * 1. Checks if casino already exists
 * 2. Calls initialize_casino_vault instruction (re-initializes Casino with new CasinoVault)
 * 3. Transfers existing funds from old vault_authority PDA to new casino_vault
 * 4. Displays balances and explorer links
 */

const anchor = require("@coral-xyz/anchor");
const {
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

// Configuration
const PROGRAM_ID = new PublicKey(
  "HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP",
);
const CLUSTER_URL = "https://api.devnet.solana.com";

async function main() {
  console.log("🏗️  Initialize CasinoVault and Migrate Funds");
  console.log("===========================================\n");

  // Load wallet
  const walletKeypairPath =
    process.argv[2] || path.join(process.env.HOME, ".config/solana/id.json");

  let wallet;
  try {
    const keypairData = JSON.parse(fs.readFileSync(walletKeypairPath, "utf-8"));
    wallet = anchor.web3.Keypair.fromSecretKey(new Uint8Array(keypairData));
  } catch (err) {
    console.error("❌ Failed to load wallet from:", walletKeypairPath);
    console.error("Usage: node initialize-casino-vault.js [keypair_path]");
    process.exit(1);
  }

  console.log(`Authority: ${wallet.publicKey.toString()}`);

  // Setup connection and provider
  const connection = new anchor.web3.Connection(CLUSTER_URL, "confirmed");
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" },
  );
  anchor.setProvider(provider);

  // Load IDL
  const idlPath = path.join(
    __dirname,
    "solana-playground-deploy/target/idl/vault.json",
  );
  let idl;
  try {
    idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
  } catch (err) {
    console.error("❌ Failed to load IDL from:", idlPath);
    console.error(
      "Make sure you have built the program first: cd solana-playground-deploy && anchor build",
    );
    process.exit(1);
  }

  const program = new anchor.Program(idl, PROGRAM_ID, provider);

  // Derive PDAs
  const [casino] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino")],
    PROGRAM_ID,
  );

  const [casinoVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino-vault"), casino.toBuffer()],
    PROGRAM_ID,
  );

  const [vaultAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault-authority"), casino.toBuffer()],
    PROGRAM_ID,
  );

  console.log(`Program ID: ${PROGRAM_ID.toString()}`);
  console.log(`Casino PDA: ${casino.toString()}`);
  console.log(`Casino Vault (NEW): ${casinoVault.toString()}`);
  console.log(`Vault Authority (OLD): ${vaultAuthority.toString()}`);
  console.log(`\n`);

  // Check if casino already exists
  try {
    const casinoAccount = await program.account.casino.fetch(casino);
    console.log("⚠️  Casino already exists!");
    console.log("This script will RE-INITIALIZE the casino.");
    console.log(
      "This should ONLY be done if you are upgrading from the old vault_authority architecture.",
    );
    console.log(
      "\nExisting casino authority:",
      casinoAccount.authority.toString(),
    );

    if (casinoAccount.authority.toString() !== wallet.publicKey.toString()) {
      console.error("\n❌ ERROR: You are not the casino authority!");
      console.error(`Expected: ${wallet.publicKey.toString()}`);
      console.error(`Got: ${casinoAccount.authority.toString()}`);
      process.exit(1);
    }
  } catch (err) {
    if (err.message.includes("Account does not exist")) {
      console.log("✅ Casino does not exist yet - will create fresh");
    } else {
      console.error("❌ Error checking casino:", err.message);
      process.exit(1);
    }
  }

  // Check balances before
  const authorityBalanceBefore = await connection.getBalance(wallet.publicKey);
  const vaultAuthorityBalanceBefore =
    await connection.getBalance(vaultAuthority);

  console.log("\n📊 Balances Before:");
  console.log(
    `  Authority: ${(authorityBalanceBefore / LAMPORTS_PER_SOL).toFixed(4)} SOL`,
  );
  console.log(
    `  Old Vault Authority: ${(vaultAuthorityBalanceBefore / LAMPORTS_PER_SOL).toFixed(4)} SOL`,
  );

  // Check if casino vault already exists
  try {
    const casinoVaultInfo = await connection.getAccountInfo(casinoVault);
    if (casinoVaultInfo) {
      console.log("\n⚠️  Casino Vault already exists!");
      const casinoVaultBalance = await connection.getBalance(casinoVault);
      console.log(
        `  Current balance: ${(casinoVaultBalance / LAMPORTS_PER_SOL).toFixed(4)} SOL`,
      );
      console.log(
        "\nSkipping initialization. If you need to re-initialize, close the account first.",
      );
      process.exit(0);
    }
  } catch (err) {
    // Account doesn't exist - continue
  }

  console.log("\n⏳ Initializing Casino Vault...");

  try {
    const tx = await program.methods
      .initializeCasinoVault(wallet.publicKey)
      .accounts({
        casino: casino,
        casinoVault: casinoVault,
        vaultAuthority: vaultAuthority,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Casino Vault initialized!");
    console.log(`Signature: ${tx}`);
    console.log(`https://explorer.solana.com/tx/${tx}?cluster=devnet\n`);
  } catch (err) {
    console.error("❌ Initialization failed:", err.message);
    if (err.logs) {
      console.error("\nProgram logs:");
      err.logs.forEach((log) => console.error(log));
    }
    process.exit(1);
  }

  // Wait for confirmation
  await new Promise((resolve) => setTimeout(resolve, 2000));

  // Transfer funds from old vault_authority to new casino_vault
  if (vaultAuthorityBalanceBefore > 0) {
    console.log(
      `\n💸 Transferring ${(vaultAuthorityBalanceBefore / LAMPORTS_PER_SOL).toFixed(4)} SOL from old vault to new vault...`,
    );

    try {
      const transferTx = await connection.sendTransaction(
        new anchor.web3.Transaction().add(
          SystemProgram.transfer({
            fromPubkey: vaultAuthority,
            toPubkey: casinoVault,
            lamports: vaultAuthorityBalanceBefore,
          }),
        ),
        [wallet],
        { skipPreflight: false },
      );

      await connection.confirmTransaction(transferTx);

      console.log("✅ Transfer complete!");
      console.log(`Signature: ${transferTx}`);
      console.log(
        `https://explorer.solana.com/tx/${transferTx}?cluster=devnet\n`,
      );
    } catch (err) {
      console.error("❌ Transfer failed:", err.message);
      console.error(
        "NOTE: Old vault_authority is a PDA, so direct transfer will fail.",
      );
      console.error(
        "You will need to use withdraw_casino_funds instruction from the old vault first,",
      );
      console.error(
        "then manually send SOL to the new casino_vault address.\n",
      );
      console.error(`Manual transfer command:`);
      console.error(
        `solana transfer ${casinoVault.toString()} ${(vaultAuthorityBalanceBefore / LAMPORTS_PER_SOL).toFixed(2)} --url devnet\n`,
      );
    }
  }

  // Check final balances
  const casinoVaultBalanceAfter = await connection.getBalance(casinoVault);
  const vaultAuthorityBalanceAfter =
    await connection.getBalance(vaultAuthority);

  console.log("\n📊 Balances After:");
  console.log(
    `  New Casino Vault: ${(casinoVaultBalanceAfter / LAMPORTS_PER_SOL).toFixed(4)} SOL`,
  );
  console.log(
    `  Old Vault Authority: ${(vaultAuthorityBalanceAfter / LAMPORTS_PER_SOL).toFixed(4)} SOL`,
  );

  console.log("\n✅ Migration Complete!");
  console.log("\n📋 Next Steps:");
  console.log("1. Update processor service to restart and pick up new program");
  console.log("2. Test bet placement to verify casino vault receives funds");
  console.log(
    '3. Monitor processor logs for "ExternalAccountLamportSpend" errors (should be gone)',
  );
  console.log(
    "4. Update fund-casino-vault.js to use new casino_vault address\n",
  );
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
