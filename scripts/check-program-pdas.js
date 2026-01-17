const { PublicKey } = require("@solana/web3.js");

const OLD_PROGRAM = "Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL";
const NEW_PROGRAM = "HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP";

// Derive casino PDA for both programs
const [casinoOld] = PublicKey.findProgramAddressSync(
  [Buffer.from("casino")],
  new PublicKey(OLD_PROGRAM),
);

const [casinoNew] = PublicKey.findProgramAddressSync(
  [Buffer.from("casino")],
  new PublicKey(NEW_PROGRAM),
);

console.log("=== Casino PDA Comparison ===");
console.log("OLD Program ID:", OLD_PROGRAM);
console.log("OLD Casino PDA:", casinoOld.toBase58());
console.log("");
console.log("NEW Program ID:", NEW_PROGRAM);
console.log("NEW Casino PDA:", casinoNew.toBase58());
console.log("");
console.log("UI Shows Casino:", "Hk51rWkSqnZfqAqQtvuLnZKxzT79fS5n5ksAJ86TJpCj");
console.log("");

if (casinoOld.toBase58() === "Hk51rWkSqnZfqAqQtvuLnZKxzT79fS5n5ksAJ86TJpCj") {
  console.log("❌ UI IS USING OLD PROGRAM!");
  console.log("");
  console.log("The frontend did not pick up the .env change.");
  console.log("Solutions:");
  console.log("1. Hard refresh browser (Cmd+Shift+R)");
  console.log("2. Clear localStorage and refresh");
  console.log("3. Rebuild: cd test-ui && npm run build && npm run dev");
} else if (
  casinoNew.toBase58() === "Hk51rWkSqnZfqAqQtvuLnZKxzT79fS5n5ksAJ86TJpCj"
) {
  console.log("✅ UI IS USING NEW PROGRAM!");
  console.log("");
  console.log("Casino needs to be initialized for the new program.");
} else {
  console.log("⚠️  Unknown - PDA does not match either program");
}
