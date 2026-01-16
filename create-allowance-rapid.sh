#!/bin/bash
# Create native SOL allowance with precise timestamp matching

set -e

echo "🔧 Creating Native SOL Allowance with precise timestamp..."

cat > /tmp/precise_allowance.rs << 'RUST_EOF'
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user_json = std::fs::read_to_string("/Users/reece/code/projects/atomik-wallet/test-user-keypair.json")?;
    let user_bytes: Vec<u8> = serde_json::from_str(&user_json)?;
    let user = Keypair::from_bytes(&user_bytes)?;
    
    let program_id = Pubkey::from_str("HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4")?;
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
    println!();
    
    // Try multiple timestamps around current time
    let slot = client.get_slot()?;
    let base_time = client.get_block_time(slot)?;
    
    // Try a range of timestamps (current time +/- 15 seconds)
    for offset in -5..20 {
        let timestamp = base_time + offset;
        
        let (allowance, _) = Pubkey::find_program_address(
            &[
                b"allowance",
                user.pubkey().as_ref(),
                casino.as_ref(),
                &timestamp.to_le_bytes(),
            ],
            &program_id,
        );
        
        println!("Attempt with timestamp {} (offset +{})...", timestamp, offset);
        println!("  Allowance PDA: {}", allowance);
        
        // Build instruction
        let amount: u64 = 1_000_000_000;
        let duration: i64 = 86400;
        
        let mut data = vec![100, 169, 165, 25, 25, 255, 11, 45];
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&duration.to_le_bytes());
        data.extend_from_slice(system_program::ID.as_ref());
        
        // Add priority fee to get confirmed faster
        let priority_ix = ComputeBudgetInstruction::set_compute_unit_price(1000);
        
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
            &[priority_ix, approve_ix],
            Some(&user.pubkey()),
            &[&user],
            recent_blockhash,
        );
        
        // Try sending with skip_preflight
        let config = solana_client::rpc_config::RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        };
        
        match client.send_transaction_with_config(&tx, config) {
            Ok(sig) => {
                println!("  Transaction sent: {}", sig);
                
                // Wait and check if it actually succeeded
                std::thread::sleep(std::time::Duration::from_millis(800));
                
                let mut confirmed_success = false;
                for _ in 0..8 {
                    match client.get_signature_status(&sig) {
                        Ok(Some(Ok(_))) => {
                            confirmed_success = true;
                            break;
                        }
                        Ok(Some(Err(_))) => {
                            println!("  Transaction failed (wrong timestamp)");
                            break;
                        }
                        _ => std::thread::sleep(std::time::Duration::from_millis(300)),
                    }
                }
                
                if confirmed_success {
                    println!();
                    println!("✅ SUCCESS! Allowance created!");
                    println!();
                    println!("Signature: {}", sig);
                    println!("Allowance PDA: {}", allowance);
                    println!("Timestamp used: {}", timestamp);
                    println!();
                    println!("📝 Update processor:");
                    println!("   File: services/processor/src/solana_tx.rs");
                    println!("   Change allowance to: {}", allowance);
                    return Ok(());
                } else {
                    continue;
                }
            }
            Err(e) => {
                println!("  Failed to send: {:?}", e);
                continue;
            }
        }
    }
    
    Err("All attempts failed. The Clock timestamp is changing too fast.".into())
}
RUST_EOF

echo "⏳ Compiling..."
mkdir -p /tmp/precise-allowance
cd /tmp/precise-allowance

cat > Cargo.toml << 'TOML_EOF'
[package]
name = "precise-allowance"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-client = "2.1"
solana-sdk = "2.1"
serde_json = "1.0"
TOML_EOF

mkdir -p src
cp /tmp/precise_allowance.rs src/main.rs

cargo build --release --quiet
./target/release/precise-allowance
