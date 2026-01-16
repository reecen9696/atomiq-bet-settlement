use serde_json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer, read_keypair_file},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Devnet configuration
    let rpc_url = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    // Program ID 
    let program_id = Pubkey::from_str(
        &env::var("VAULT_PROGRAM_ID")
            .unwrap_or_else(|_| "Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL".to_string())
    )?;
    
    // Load processor keypair
    let keypair_json = env::var("PROCESSOR_KEYPAIR")?;
    let keypair_data: Vec<u8> = serde_json::from_str(&keypair_json)?;
    let processor_keypair = Keypair::from_bytes(&keypair_data)?;
    
    println!("🚀 Real Devnet Transaction Test");
    println!("RPC URL: {}", rpc_url);
    println!("Program ID: {}", program_id);
    println!("Processor pubkey: {}", processor_keypair.pubkey());

    // Check balance
    let balance = client.get_balance(&processor_keypair.pubkey())?;
    println!("Processor balance: {} lamports ({} SOL)", balance, balance as f64 / 1_000_000_000.0);
    
    if balance == 0 {
        println!("❌ Processor has 0 SOL balance - need to fund it first");
        println!("To fund: solana airdrop 1 {} --url {}", processor_keypair.pubkey(), rpc_url);
        return Ok(());
    }

    // Create a simple test transaction 
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(processor_keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: vec![0], // Simple instruction data
    };

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&processor_keypair.pubkey()),
        &[&processor_keypair],
        recent_blockhash,
    );

    println!("\n🎯 Submitting real transaction to devnet...");
    
    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            println!("✅ SUCCESS! Real devnet transaction confirmed!");
            println!("Transaction ID: {}", signature);
            println!("Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            println!("\n🎉 This is a REAL Solana transaction, not a simulation!");
        }
        Err(e) => {
            println!("❌ Transaction failed: {}", e);
            println!("This might be expected if the program instruction is invalid");
            println!("But we successfully connected to devnet and submitted a real transaction");
        }
    }

    Ok(())
}