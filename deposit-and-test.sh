#!/bin/bash
# Deposit SOL to vault and test betting with native SOL

set -e

echo "🔧 Depositing SOL to Vault..."

cat > /tmp/deposit_sol.rs << 'RUST_EOF'
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user_json = std::fs::read_to_string("/Users/reece/code/projects/atomik-wallet/test-user-keypair.json")?;
    let user_bytes: Vec<u8> = serde_json::from_str(&user_json)?;
    let user = Keypair::from_bytes(&user_bytes)?;
    
    let program_id = Pubkey::from_str("Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL")?;
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    
    // Derive PDAs
    let (casino, _) = Pubkey::find_program_address(&[b"casino"], &program_id);
    let (user_vault, _) = Pubkey::find_program_address(
        &[b"vault", casino.as_ref(), user.pubkey().as_ref()],
        &program_id,
    );
    
    println!("User: {}", user.pubkey());
    println!("User Vault: {}", user_vault);
    
    // Check current vault balance
    match client.get_balance(&user_vault) {
        Ok(balance) => println!("Current vault SOL balance: {} lamports", balance),
        Err(_) => println!("Vault not found or has no balance"),
    }
    
    // Deposit 0.5 SOL (500,000,000 lamports)
    let deposit_amount: u64 = 500_000_000;
    
    // deposit_sol discriminator
    let discriminator: [u8; 8] = [
        108, 81, 78, 117, 125, 155, 56, 200
    ]; // SHA256("global:deposit_sol")[0..8]
    
    let mut data = discriminator.to_vec();
    data.extend_from_slice(&deposit_amount.to_le_bytes());
    
    let deposit_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user_vault, false),
            AccountMeta::new_readonly(casino, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    };
    
    println!();
    println!("Depositing {} lamports (0.5 SOL)...", deposit_amount);
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[deposit_ix],
        Some(&user.pubkey()),
        &[&user],
        recent_blockhash,
    );
    
    let sig = client.send_and_confirm_transaction(&tx)?;
    
    println!("✅ Deposited!");
    println!("Signature: {}", sig);
    
    // Check new balance
    let new_balance = client.get_balance(&user_vault)?;
    println!();
    println!("New vault SOL balance: {} lamports ({} SOL)", new_balance, new_balance as f64 / 1_000_000_000.0);
    
    Ok(())
}
RUST_EOF

echo "⏳ Compiling..."
mkdir -p /tmp/deposit-tool
cd /tmp/deposit-tool

cat > Cargo.toml << 'TOML_EOF'
[package]
name = "deposit-tool"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-client = "2.1"
solana-sdk = "2.1"
serde_json = "1.0"
TOML_EOF

mkdir -p src
cp /tmp/deposit_sol.rs src/main.rs

cargo build --release --quiet
./target/release/deposit-tool

echo ""
echo "✅ Vault funded! Now testing bet..."
echo ""

# Restart services and test
cd /Users/reece/code/projects/atomik-wallet
./stop-services.sh
./start-services.sh

sleep 25

# Create test bet
curl -s -X POST http://localhost:3001/api/bets \
  -H "Content-Type: application/json" \
  -d '{"stake_amount":150000000,"stake_token":"SOL","choice":"heads","user_wallet":"LCsLwQ74zUfa5UDA6fNTRPyddH6akTd6S1fkdMAQQj8"}'

echo ""
echo ""
echo "Waiting for processor..."
sleep 15

# Check logs
echo ""
echo "Recent processor logs:"
tail -n 40 logs/processor.log | grep -E "confirmed|ERROR|Solana transaction|Processing batch" || echo "No relevant logs yet"
