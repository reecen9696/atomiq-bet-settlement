# Keypairs Directory

This directory contains Solana keypairs used by the transaction processor. These files are git-ignored for security.

## Required Keypairs

1. **processor-keypair.json** - Main processor keypair (authorized to execute settlements)
2. **test-keypair.json** - Test user keypair for development
3. **test-user-keypair.json** - Additional test user

## Generate Keypairs

```bash
# Generate processor keypair
solana-keygen new --outfile processor-keypair.json --no-bip39-passphrase

# Generate test keypairs
solana-keygen new --outfile test-keypair.json --no-bip39-passphrase
solana-keygen new --outfile test-user-keypair.json --no-bip39-passphrase
```

## Fund Keypairs (DevNet)

```bash
# Fund processor (needs SOL for transaction fees)
solana airdrop 2 $(solana-keygen pubkey processor-keypair.json) --url devnet

# Fund test users
solana airdrop 5 $(solana-keygen pubkey test-keypair.json) --url devnet
solana airdrop 5 $(solana-keygen pubkey test-user-keypair.json) --url devnet
```

## Get Public Keys

```bash
# Processor pubkey (add this to casino.processor in smart contract)
solana-keygen pubkey processor-keypair.json

# Test user pubkeys
solana-keygen pubkey test-keypair.json
solana-keygen pubkey test-user-keypair.json
```

## Security Notes

⚠️ **NEVER commit keypairs to git!**

- These files contain private keys
- The .gitignore is configured to exclude all .json, .key, and .keypair files in this directory
- For production, use HSM/KMS instead of file-based keypairs
- Rotate keypairs regularly
- Keep backups in a secure location (not in the git repository)

## Production Considerations

For production deployments:

1. Generate keypairs on secure, air-gapped machines
2. Store in Hardware Security Module (HSM) or Key Management Service (KMS)
3. Never store in plaintext files
4. Implement key rotation procedures
5. Use separate keypairs for each environment (dev/staging/prod)
