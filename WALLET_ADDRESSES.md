# Hardcoded Wallet Addresses

## Processor Wallet
**Purpose**: Executes bets on-chain, pays transaction fees
**Keypair**: `test-keypair.json`
**Address**: Run to get address:
```bash
solana-keygen pubkey test-keypair.json
```
**Required Balance**: 2 SOL (devnet)

## Test User Wallet
**Purpose**: Testing vault deposits/withdrawals
**Keypair**: `test-user-keypair.json`
**Address**: Run to get address:
```bash
solana-keygen pubkey test-user-keypair.json
```
**Required Balance**: 0.5 SOL (devnet)

## Get Wallet Addresses

```bash
cd /Users/reece/code/projects/atomik-wallet

# Processor wallet address
echo "Processor Wallet:"
solana-keygen pubkey test-keypair.json

echo ""

# Test user wallet address
echo "Test User Wallet:"
solana-keygen pubkey test-user-keypair.json
```

## Fund Wallets

1. Run the commands above to get both addresses
2. Go to https://faucet.solana.com
3. Paste each address and request devnet SOL
4. Verify balances:

```bash
# Check processor balance
solana balance $(solana-keygen pubkey test-keypair.json) --url devnet

# Check test user balance
solana balance $(solana-keygen pubkey test-user-keypair.json) --url devnet
```

## Configuration

Both keypairs are already configured:
- ✅ Processor uses `test-keypair.json` (configured in `services/processor/.env`)
- ✅ Test user uses `test-user-keypair.json` (for future integration tests)
