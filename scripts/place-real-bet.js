#!/usr/bin/env node
// Place a real bet with actual wallet addresses and PDAs

const { PublicKey } = require("@solana/web3.js");

const PROGRAM_ID = new PublicKey(
  "BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ",
);
const USER_WALLET = new PublicKey(
  "LCsLwQ74zUfa5UDA6fNTRPyddH6akTd6S1fkdMAQQj8",
);

// Derive Casino PDA
const [casinoPDA] = PublicKey.findProgramAddressSync(
  [Buffer.from("casino")],
  PROGRAM_ID,
);

// Derive User Vault PDA
const [vaultPDA] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), casinoPDA.toBuffer(), USER_WALLET.toBuffer()],
  PROGRAM_ID,
);

console.log("🎲 Real Bet Configuration");
console.log("========================");
console.log("Program ID:", PROGRAM_ID.toBase58());
console.log("User Wallet:", USER_WALLET.toBase58());
console.log("Casino PDA:", casinoPDA.toBase58());
console.log("Vault PDA:", vaultPDA.toBase58());
console.log("");

// Place bet via API
const fetch = require("node-fetch");

const betData = {
  user_wallet: USER_WALLET.toBase58(),
  vault_address: vaultPDA.toBase58(),
  allowance_pda: null, // Will be created when we test with allowances
  stake_amount: 100000000, // 0.1 SOL
  stake_token: "SOL",
  choice: "heads",
};

console.log("📤 Placing bet...");
console.log("Stake:", betData.stake_amount / 1e9, "SOL");
console.log("Choice:", betData.choice);
console.log("");

fetch("http://localhost:3001/api/bets", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify(betData),
})
  .then((res) => res.json())
  .then((data) => {
    console.log("✅ Bet created:");
    console.log(JSON.stringify(data, null, 2));
    console.log("");
    console.log("Bet ID:", data.bet?.bet_id || data.bet_id);
    console.log("");
    console.log("🔍 Monitor with:");
    console.log(
      `curl http://localhost:3001/api/bets/${data.bet?.bet_id || data.bet_id} | jq`,
    );
    console.log("");
    console.log("⏳ Waiting for processor to pick up bet...");

    const betId = data.bet?.bet_id || data.bet_id;
    let attempts = 0;
    const maxAttempts = 20;

    const checkStatus = () => {
      attempts++;
      fetch(`http://localhost:3001/api/bets/${betId}`)
        .then((res) => res.json())
        .then((bet) => {
          const status = bet.status || bet.bet?.status;
          process.stdout.write(
            `\r   [${attempts}/${maxAttempts}] Status: ${status.padEnd(12)}`,
          );

          if (status === "completed" || status === "settled") {
            console.log(" ✅\n");
            console.log("🎉 Bet completed!\n");
            console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            console.log("Bet ID:", bet.bet_id);
            console.log("Status:", bet.status);
            console.log("Won:", bet.won ? "🏆 YES" : "😔 NO");
            console.log("Stake:", bet.stake_amount / 1e9, "SOL");
            console.log(
              "Payout:",
              bet.payout_amount ? bet.payout_amount / 1e9 + " SOL" : "None",
            );
            console.log("Result:", bet.result || "N/A");
            console.log("Solana TX:", bet.solana_tx_id || "None");
            console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

            if (bet.solana_tx_id && !bet.solana_tx_id.startsWith("SIM_")) {
              console.log("🔗 View on Explorer:");
              console.log(
                `https://explorer.solana.com/tx/${bet.solana_tx_id}?cluster=devnet\n`,
              );
            } else if (
              bet.solana_tx_id &&
              bet.solana_tx_id.startsWith("SIM_")
            ) {
              console.log(
                "⚠️  This was a SIMULATED transaction (not on-chain)",
              );
              console.log("   The processor fell back to simulation mode.\n");
            }

            process.exit(0);
          } else if (status === "failed") {
            console.log(" ❌\n");
            console.log("Error:", bet.last_error_message || "Unknown error");
            process.exit(1);
          } else if (attempts >= maxAttempts) {
            console.log(
              "\n\n⏳ Bet still processing after",
              maxAttempts,
              "attempts",
            );
            console.log(
              `Check manually: curl http://localhost:3001/api/bets/${betId} | jq\n`,
            );
            process.exit(0);
          } else {
            setTimeout(checkStatus, 3000);
          }
        })
        .catch((err) => {
          console.error("\n❌ Error checking bet status:", err.message);
          process.exit(1);
        });
    };

    setTimeout(checkStatus, 3000);
  })
  .catch((err) => {
    console.error("❌ Error placing bet:", err.message);
    process.exit(1);
  });
