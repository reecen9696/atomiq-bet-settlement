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
    // Devnet configuration
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Program ID 
    let program_id = Pubkey::from_str("HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4")?;
    
    // Load user keypair (we'll use a test one)
    let user_keypair = Keypair::new();
    println!("User public key: {}", user_keypair.pubkey());

    // Create a simple test transaction to our program
    let instruction = Instruction::new_with_bincode(
        program_id,
        &[1u8], // Simple data
        vec![
            AccountMeta::new(user_keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&user_keypair.pubkey()),
        &[&user_keypair],
        recent_blockhash,
    );

    println!("Transaction created successfully!");
    println!("Transaction ID would be: {:?}", transaction.signatures[0]);
    println!("This is a simulation - we didn't send it to avoid needing SOL balance");
    println!("Program ID being used: {}", program_id);
    println!("RPC endpoint: {}", rpc_url);

    Ok(())
}