#!/bin/bash
# Create allowance for native SOL betting with proper timestamp handling

set -e

echo "🔧 Creating Native SOL Allowance..."

cat > /tmp/native_sol_allowance.rs << 'RUST_EOF'
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
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
    println!("Casino: {}", casino);
    println!("User Vault: {}", user_vault);
    println!();
    
    // Get current slot and estimate timestamp
    // Add buffer since Clock::get() will be called when tx executes (few seconds later)
    let slot = client.get_slot()?;
    let block_time = client.get_block_time(slot)? + 3; // Add 3 second buffer
    
    println!("Current block time + buffer: {}", block_time);
    println!();
    
    // Derive allowance with buffered timestamp
    let (allowance_estimate, _) = Pubkey::find_program_address(
        &[
            b"allowance",
            user.pubkey().as_ref(),
            casino.as_ref(),
            &block_time.to_le_bytes(),
        ],
        &program_id,
    );
    
    println!("Estimated Allowance PDA: {}", allowance_estimate);
    
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
            AccountMeta::new(allowance_estimate, false), // Use estimated allowance
            AccountMeta::new(rate_limiter, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    };
    
    println!("Approving 1 SOL allowance for 24 hours with native SOL (System::id())...");
    println!("⚠️  Skipping preflight to avoid timestamp mismatch...");
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[approve_ix],
        Some(&user.pubkey()),
        &[&user],
        recent_blockhash,
    );
    
    // Send with skip_preflight since we can't predict exact timestamp
    let config = solana_client::rpc_config::RpcSendTransactionConfig {
        skip_preflight: true,
        ..Default::default()
    };
    
    let sig = client.send_transaction_with_config(&tx, config)?;
    println!("Transaction sent: {}", sig);
    println!("Waiting for confirmation...");
    
    // Wait for confirmation
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    let mut confirmed = false;
    for _ in 0..30 {
        match client.get_signature_status(&sig)? {
            Some(Ok(())) => {
                confirmed = true;
                break;
            }
            Some(Err(e)) => {
                return Err(format!("Transaction failed: {:?}", e).into());
            }
            None => {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
    
    if !confirmed {
        return Err("Transaction not confirmed after 15 seconds".into());
    }
    
    println!("✅ Native SOL Allowance Created!");
    println!("Signature: {}", sig);
    
    // Get the block time from the confirmed transaction
    let tx_status = client.get_transaction(&sig, UiTransactionEncoding::Json)?;
    let tx_block_time = tx_status.block_time.expect("Block time should be present");
    
    println!("Transaction block time: {}", tx_block_time);
    
    // Now derive the allowance address using the actual block time
    let (allowance, _) = Pubkey::find_program_address(
        &[
            b"allowance",
            user.pubkey().as_ref(),
            casino.as_ref(),
            &tx_block_time.to_le_bytes(),
        ],
        &program_id,
    );
    
    println!();
    println!("✅ Allowance PDA: {}", allowance);
    println!();
    println!("📝 Next step: Update processor to use this allowance address");
    println!("   File: services/processor/src/solana_tx.rs");
    println!("   Line: ~55");
    println!("   Change: let allowance = Pubkey::from_str(\"{}\")", allowance);
    
    Ok(())
}
RUST_EOF

echo "⏳ Compiling..."
mkdir -p /tmp/native-sol-tool
cd /tmp/native-sol-tool

cat > Cargo.toml << 'TOML_EOF'
[package]
name = "native-sol-allowance"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-client = "2.1"
solana-sdk = "2.1"
solana-transaction-status = "2.1"
serde_json = "1.0"
TOML_EOF

mkdir -p src
cp /tmp/native_sol_allowance.rs src/main.rs

cargo build --release --quiet
./target/release/native-sol-allowance
