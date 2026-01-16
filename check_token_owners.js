const { Connection, PublicKey } = require('@solana/web3.js');

async function checkTokenAccounts() {
    const connection = new Connection('https://api.devnet.solana.com');
    
    // User token account
    const userTokenAccount = new PublicKey('6krbSwyWLkeASqoAruq5S7jxSoTMkvXnSkaUhRYJuzWC');
    const userAccount = await connection.getAccountInfo(userTokenAccount);
    
    if (userAccount) {
        // SPL Token account layout: 32 bytes mint, 32 bytes owner, 8 bytes amount, etc.
        const ownerBytes = userAccount.data.slice(32, 64);
        const owner = new PublicKey(ownerBytes);
        console.log('User token account owner:', owner.toBase58());
    }
    
    // User wallet
    const userWallet = new PublicKey('LCsLwQ74zUfa5UDA6fNTRPyddH6akTd6S1fkdMAQQj8');
    console.log('Expected user wallet:', userWallet.toBase58());
    
    // Casino PDA
    const PROGRAM_ID = new PublicKey('Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL');
    const [casinoPDA] = PublicKey.findProgramAddressSync([Buffer.from('casino')], PROGRAM_ID);
    console.log('Casino PDA:', casinoPDA.toBase58());
}

checkTokenAccounts().catch(console.error);