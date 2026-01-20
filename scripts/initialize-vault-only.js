const {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
} = require("@solana/web3.js");
const { AnchorProvider, Program, web3 } = require("@project-serum/anchor");
const fs = require("fs");
const os = require("os");

// Configuration
const PROGRAM_ID = new PublicKey(
  "BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ",
);
const RPC_URL = "https://api.devnet.solana.com";

async function main() {
  console.log("🔧 Initializing Casino Vault Only...\n");

  // Load wallet
  const keypairPath = `${os.homedir()}/.config/solana/id.json`;
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  const keypair = web3.Keypair.fromSecretKey(new Uint8Array(keypairData));

  console.log(`Authority: ${keypair.publicKey.toString()}`);

  // Setup connection and provider
  const connection = new Connection(RPC_URL, "confirmed");
  const wallet = {
    publicKey: keypair.publicKey,
    signTransaction: async (tx) => {
      tx.sign(keypair);
      return tx;
    },
    signAllTransactions: async (txs) => {
      txs.forEach((tx) => tx.sign(keypair));
      return txs;
    },
  };
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });

  // Load IDL (you'll need to generate this from your program)
  // For now, we'll build the transaction manually

  // Derive PDAs
  const [casinoPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino")],
    PROGRAM_ID,
  );

  const [casinoVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("casino-vault"), casinoPda.toBuffer()],
    PROGRAM_ID,
  );

  console.log(`\nCasino PDA: ${casinoPda.toString()}`);
  console.log(`Casino Vault PDA: ${casinoVaultPda.toString()}\n`);

  // Check if casino exists
  const casinoAccount = await connection.getAccountInfo(casinoPda);
  if (!casinoAccount) {
    console.error("❌ Casino account not found! Initialize casino first.");
    process.exit(1);
  }
  console.log("✅ Casino account exists");

  // Check if vault already exists
  const vaultAccount = await connection.getAccountInfo(casinoVaultPda);
  if (vaultAccount) {
    console.log("⚠️  Casino vault already initialized!");
    const balance = await connection.getBalance(casinoVaultPda);
    console.log(`Current balance: ${balance / 1e9} SOL`);
    process.exit(0);
  }

  console.log("\n📝 Building initialize_vault_only instruction...");
  console.log("Note: You need to build and deploy the updated program first!");
  console.log("\nSteps to complete:");
  console.log("1. Build: cd solana-playground-deploy && anchor build");
  console.log(
    "2. Deploy: solana program deploy target/deploy/vault.so --program-id <KEYPAIR>",
  );
  console.log("3. Then run this script again");

  // For reference, the instruction would look like this:
  console.log("\nInstruction accounts:");
  console.log(`  casino: ${casinoPda.toString()}`);
  console.log(`  casino_vault: ${casinoVaultPda.toString()} (will be created)`);
  console.log(`  authority: ${keypair.publicKey.toString()}`);
  console.log(`  system_program: ${SystemProgram.programId.toString()}`);
}

main().catch(console.error);
