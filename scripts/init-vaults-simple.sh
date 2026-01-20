#!/bin/bash

# Simple Vault Initialization using Rust script
set -e

echo "🔧 Initializing Vaults..."
echo ""

# Create a simple Rust program to do the initialization
cat > /tmp/init_vaults.rs << 'RUST_EOF'
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
    // Change to project directory
    std::env::set_current_dir("/Users/reece/code/projects/atomik-wallet")?;
    
    // Load keypairs
    let processor_json = std::fs::read_to_string("../keys/test-keypair.json")?;
    let processor_bytes: Vec<u8> = serde_json::from_str(&processor_json)?;
    let processor = Keypair::from_bytes(&processor_bytes)?;
    
    let user_json = std::fs::read_to_string("../keys/test-user-keypair.json")?;
    let user_bytes: Vec<u8> = serde_json::from_str(&user_json)?;
    let user = Keypair::from_bytes(&user_bytes)?;
    
    let program_id = Pubkey::from_str("Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL")?;
    
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    
    println!("👤 User: {}", user.pubkey());
    println!("🔑 Processor: {}", processor.pubkey());
    println!("📝 Program: {}", program_id);
    println!();
    
    // 1. Derive casino PDA (must be initialized first!)
    let (casino, casino_bump) = Pubkey::find_program_address(
        &[b"casino"],
        &program_id,
    );
    println!("1️⃣ Casino PDA: {} (bump: {})", casino, casino_bump);
    // Check if already initialized
    match client.get_account(&casino) {
        Ok(_) => {
            println!("   ℹ️  Already initialized");
        }
        Err(_) => {
            println!("   Creating casino...");
            
            // Derive vault_authority PDA
            let (vault_authority, _) = Pubkey::find_program_address(
                &[b"vault-authority", casino.as_ref()],
                &program_id,
            );
            
            // initialize_casino_vault discriminator: [143, 226, 254, 191, 118, 163, 213, 51]
            let mut data = vec![143, 226, 254, 191, 118, 163, 213, 51];
            data.extend_from_slice(processor.pubkey().as_ref());
            
            let init_casino_ix = Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(casino, false),
                    AccountMeta::new_readonly(vault_authority, false),
                    AccountMeta::new(processor.pubkey(), true),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
                data,
            };
            
            let recent_blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[init_casino_ix],
                Some(&processor.pubkey()),
                &[&processor],
                recent_blockhash,
            );
            
            let sig = client.send_and_confirm_transaction(&tx)?;
            println!("   ✅ Created: {}", sig);
        }
    }
    
    //2. Derive user vault PDA  
    let (user_vault, user_bump) = Pubkey::find_program_address(
        &[b"vault", casino.as_ref(), user.pubkey().as_ref()],
        &program_id,
    );
    println!();
    println!("2️⃣ User Vault PDA: {} (bump: {})", user_vault, user_bump);
    
    // Check if already initialized
    match client.get_account(&user_vault) {
        Ok(_) => {
            println!("   ℹ️  Already initialized");
        }
        Err(_) => {
            println!("   Creating user vault...");
            
            // initialize_vault discriminator: [48, 191, 163, 44, 71, 129, 63, 164]
            let init_vault_ix = Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(user_vault, false),
                    AccountMeta::new_readonly(casino, false),
                    AccountMeta::new(user.pubkey(), true),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
                data: vec![48, 191, 163, 44, 71, 129, 63, 164],
            };
            
            let recent_blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[init_vault_ix],
                Some(&user.pubkey()),
                &[&user],
                recent_blockhash,
            );
            
            let sig = client.send_and_confirm_transaction(&tx)?;
            println!("   ✅ Created: {}", sig);
        }
    }
    
    // 3. Approve Allowance
    // Note: Seeds include timestamp, so we'll use current timestamp
    println!();
    println!("3️⃣ Approving allowance...");
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    
    let (allowance, allowance_bump) = Pubkey::find_program_address(
        &[
            b"allowance",
            user.pubkey().as_ref(),
            casino.as_ref(),
            &now.to_le_bytes(),
        ],
        &program_id,
    );
    println!("   Allowance PDA: {} (bump: {})", allowance, allowance_bump);
    
    // Derive rate_limiter PDA
    let (rate_limiter, _) = Pubkey::find_program_address(
        &[b"rate-limiter", user.pubkey().as_ref()],
        &program_id,
    );
    
    println!("   Approving 1 SOL for 24 hours...");
    
    // approve_allowance discriminator: [100, 169, 165, 25, 25, 255, 11, 45]
    let amount: u64 = 1_000_000_000; // 1 SOL
    let duration: i64 = 86400; // 24 hours
    // Use System::id() for native SOL (not wrapped SOL mint)
    let native_mint = system_program::ID;
    
    let mut data = vec![100, 169, 165, 25, 25, 255, 11, 45];
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&duration.to_le_bytes());
    data.extend_from_slice(native_mint.as_ref());
    
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
    
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!("   ✅ Approved: {}", sig),
        Err(e) => println!("   ⚠️  Error: {}", e),
    }
    
    println!();
    println!("✅ All vaults initialized!");
    println!();
    println!("Now run: ./test-onchain.sh");
    
    Ok(())
}
RUST_EOF

# Compile and run
echo "⏳ Compiling initialization tool..."
rustc --edition 2021 \
    --extern solana_client \
    --extern solana_sdk \
    --extern serde_json \
    -L dependency=/Users/reece/.cargo/registry/target/release/deps \
    -o /tmp/init_vaults \
    /tmp/init_vaults.rs 2>/dev/null || {
        echo "Compiling with cargo..."
        cd /Users/reece/code/projects/atomik-wallet
        mkdir -p /tmp/vault-init
        cd /tmp/vault-init
        
        cat > Cargo.toml << 'TOML_EOF'
[package]
name = "vault-init"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-client = "2.1"
solana-sdk = "2.1"
serde_json = "1.0"
TOML_EOF
        
        mkdir -p src
        cp /tmp/init_vaults.rs src/main.rs
        
        cargo build --release --quiet
        /tmp/vault-init/target/release/vault-init
        exit 0
    }

/tmp/init_vaults
