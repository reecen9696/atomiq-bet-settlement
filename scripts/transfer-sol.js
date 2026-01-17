#!/usr/bin/env node

const { Connection, Keypair, LAMPORTS_PER_SOL, SystemProgram, Transaction, sendAndConfirmTransaction, PublicKey } = require('@solana/web3.js');
const fs = require('fs');
const bs58 = require('bs58');

async function main() {
  const args = process.argv.slice(2);
  if (args.length < 3) {
    console.error('Usage: node scripts/transfer-sol.js <keypair-path> <to-address> <amount-sol>');
    process.exit(1);
  }

  const [keypairPath, toAddress, amountSol] = args;

  const keypairContent = fs.readFileSync(keypairPath, 'utf-8').trim();
  const keypair = Keypair.fromSecretKey(bs58.decode(keypairContent));
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  const amount = parseFloat(amountSol) * LAMPORTS_PER_SOL;
  
  console.log('From:', keypair.publicKey.toBase58());
  console.log('To:', toAddress);
  console.log('Amount:', amountSol, 'SOL');
  
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: keypair.publicKey,
      toPubkey: new PublicKey(toAddress),
      lamports: amount,
    })
  );
  
  console.log('\n🚀 Sending transaction...');
  const signature = await sendAndConfirmTransaction(connection, tx, [keypair]);
  console.log('✅ Transfer complete!');
  console.log('Signature:', signature);
  console.log('Explorer:', `https://explorer.solana.com/tx/${signature}?cluster=devnet`);
}

main().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});
