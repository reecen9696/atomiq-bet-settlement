const { PublicKey } = require('@solana/web3.js');

const PROGRAM_ID = new PublicKey('Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL');

const [casinoPDA, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from('casino')],
    PROGRAM_ID
);

console.log('Casino PDA:', casinoPDA.toBase58());
console.log('Bump:', bump);
