const { PublicKey } = require('@solana/web3.js');

const PROGRAM_ID = new PublicKey('HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4');

const [casinoPDA, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from('casino')],
    PROGRAM_ID
);

console.log('Casino PDA:', casinoPDA.toBase58());
console.log('Bump:', bump);
