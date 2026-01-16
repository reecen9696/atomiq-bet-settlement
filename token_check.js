const { Connection, PublicKey } = require('@solana/web3.js');

async function main() {
    const connection = new Connection('https://api.devnet.solana.com');
    
    const userTokenAccount = new PublicKey('6krbSwyWLkeASqoAruq5S7jxSoTMkvXnSkaUhRYJuzWC');
    const userAccount = await connection.getAccountInfo(userTokenAccount);
    
    if (userAccount) {
        const ownerBytes = userAccount.data.slice(32, 64);
        const owner = new PublicKey(ownerBytes);
        console.log('User token account owner:', owner.toBase58());
    }
    
    const userWallet = new PublicKey('LCsLwQ74zUFA5UDA6fNTRPyddH6akTd6S1fkdMAQQj8');
    console.log('Expected user wallet:', userWallet.toBase58());
}

main().catch(console.error);