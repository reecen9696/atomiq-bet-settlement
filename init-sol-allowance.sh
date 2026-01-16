#!/bin/bash
# Create allowance for native SOL betting

set -e

echo "🔧 Creating SOL Allowance..."

cat > /tmp/sol_allowance.rs << 'RUST_EOF'
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
    let (rate_limiter, _) = Pubkey::find_program_address(
        &[b"rate-limiter", user.pubkey().as_ref()],
        &program_id,
    );
    
    println!("User: {}", user.pubkey());
    println!("Casino: {}", casino);
    println!("User Vault: {}", user_vault);
    
    // Use a fixed future timestamp so we can hardcode it in processor
    // Set to 2026-01-20 00:00:00 UTC
    let timestamp: i64 = 1737331200;
    
    let (allowance, _) = Pubkey::find_program_address(
        &[
            b"allowance",
            user.pubkey().as_ref(),
            casino.as_ref(),
            &timestamp.to_le_bytes(),
        ],
        &program_id,
    );
    
    println!("New Allowance: {}", allowance);
    println!();
    
    // Build approve_allowance with System::id() for native SOL
    let amount: u64 = 1_000_000_000; // 1 SOL
    let duration: i64 = 86400; // 24 hours
    
    let mut data = vec![100, 169, 165, 25, 25, 255, 11, 45]; // approve_allowance discriminator
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&duration.to_le_bytes());
    data.extend_from_slice(system_program::ID.as_ref()); // System::id() for native SOL
    
    let approve_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user_vault, false),
            AccountMeta::new_readonly(casino, false),
            AccountMeta::new(allowance, false),
            AccountMeta::new(rate_limiter, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    };
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[approve_ix],
        Some(&user.pubkey()),
        &[&user],
        recent_blockhash,
    );
    
    println!("Approving 1 SOL allowance for 24 hours...");
    let sig = client.send_and_confirm_transaction(&tx)?;
    
    println!("✅ SOL Allowance Created!");
    println!("Signature: {}", sig);
    println!();
    println!("Allowance PDA: {}", allowance);
    println!();
    println!("Update processor config with this allowance address.");
    
    Ok(())
}
RUST_EOF

echo "⏳ Compiling..."
mkdir -p /tmp/sol-allowance-tool
cd /tmp/sol-allowance-tool

cat > Cargo.toml << 'TOML_EOF'
[package]
name = "sol-allowance"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-client = "2.1"
solana-sdk = "2.1"
serde_json = "1.0"
TOML_EOF

mkdir -p src
cp /tmp/sol_allowance.rs src/main.rs

cargo build --release --quiet
./target/release/sol-allowance
