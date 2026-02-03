const { Connection, PublicKey, SystemProgram, Transaction, Keypair } = require('@solana/web3.js');
const fs = require('fs');

async function createAllowance() {
  // Configuration
  const RPC_URL = 'https://solana-devnet.g.alchemy.com/v2/bsLgryJK7I4nDQ4YDvr5l'\;
  const PROGRAM_ID = new PublicKey('BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ');
  const connection = new Connection(RPC_URL);
  
  // For testing, we'll use the Solana default keypair
  const keypairPath = process.env.HOME + '/.config/solana/id.json';
  let userKeypair;
  
  try {
    const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
    userKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
  } catch (e) {
    console.log('Using random keypair for testing...');
    userKeypair = Keypair.generate();
    
    // Fund the test account
    try {
      const signature = await connection.requestAirdrop(userKeypair.publicKey, 1e9);
      await connection.confirmTransaction(signature);
      console.log('Funded test account with 1 SOL');
    } catch (e) {
      console.log('Could not fund test account:', e.message);
    }
  }
  
  console.log('User:', userKeypair.publicKey.toBase58());
  
  // Derive PDAs
  const [casino] = PublicKey.findProgramAddressSync([Buffer.from('casino')], PROGRAM_ID);
  
  const [nonceRegistry] = PublicKey.findProgramAddressSync([
    Buffer.from('allowance-nonce'),
    userKeypair.publicKey.toBuffer(),
    casino.toBuffer()
  ], PROGRAM_ID);
  
  // Check if nonce registry exists
  let nonce = 0;
  try {
    const nonceAccount = await connection.getAccountInfo(nonceRegistry);
    if (nonceAccount) {
      nonce = Number(Buffer.from(nonceAccount.data).readBigUInt64LE(8));
    }
  } catch (e) {
    console.log('Creating first allowance...');
  }
  
  const [allowance] = PublicKey.findProgramAddressSync([
    Buffer.from('allowance'),
    userKeypair.publicKey.toBuffer(),
    casino.toBuffer(),
    Buffer.from(nonce.toString().padStart(16, '0'), 'hex').reverse()
  ], PROGRAM_ID);
  
  const [userVault] = PublicKey.findProgramAddressSync([
    Buffer.from('vault'),
    userKeypair.publicKey.toBuffer(),
    casino.toBuffer()
  ], PROGRAM_ID);
  
  console.log('Casino:', casino.toBase58());
  console.log('Allowance:', allowance.toBase58());
  console.log('User Vault:', userVault.toBase58());
  console.log('Nonce Registry:', nonceRegistry.toBase58());
  
  // Build approve allowance instruction
  const amount = BigInt(1_000_000_000); // 1 SOL
  const duration = BigInt(86400); // 24 hours
  
  // Instruction discriminator for approve_allowance: [100, 169, 165, 25, 25, 255, 11, 45]
  const data = Buffer.concat([
    Buffer.from([100, 169, 165, 25, 25, 255, 11, 45]),
    Buffer.from(amount.toString(16).padStart(16, '0'), 'hex').reverse(),
    Buffer.from(duration.toString(16).padStart(16, '0'), 'hex').reverse(),
    SystemProgram.programId.toBuffer() // Native SOL mint
  ]);
  
  const instruction = {
    programId: PROGRAM_ID,
    keys: [
      { pubkey: userVault, isSigner: false, isWritable: true },
      { pubkey: casino, isSigner: false, isWritable: false },
      { pubkey: allowance, isSigner: false, isWritable: true },
      { pubkey: nonceRegistry, isSigner: false, isWritable: true },
      { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
    ],
    data
  };
  
  const transaction = new Transaction().add(instruction);
  
  try {
    const signature = await connection.sendTransaction(transaction, [userKeypair], {
      skipPreflight: false,
      preflightCommitment: 'confirmed'
    });
    
    await connection.confirmTransaction(signature, 'confirmed');
    console.log('✅ Allowance created!');
    console.log('Signature:', signature);
    console.log('Allowance PDA:', allowance.toBase58());
    
    return allowance.toBase58();
  } catch (error) {
    console.error('❌ Error creating allowance:', error.message);
    if (error.logs) {
      console.error('Logs:', error.logs);
    }
    throw error;
  }
}

if (require.main === module) {
  createAllowance().catch(console.error);
}

module.exports = { createAllowance };
