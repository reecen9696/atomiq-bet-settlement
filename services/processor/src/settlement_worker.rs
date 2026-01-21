//! Settlement worker that polls blockchain API and processes settlements

use crate::{
    blockchain_client::{BlockchainClient, GameSettlementInfo},
    config::Config,
    solana_client::SolanaClientPool,
    solana_tx,
};
use anyhow::{Context, Result};
use solana_sdk::signature::{Keypair, Signer};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

pub struct SettlementWorker {
    blockchain_client: Arc<BlockchainClient>,
    solana_client: Arc<SolanaClientPool>,
    processor_keypair: Arc<Keypair>,
    config: Config,
}

impl SettlementWorker {
    pub fn new(
        blockchain_client: Arc<BlockchainClient>,
        solana_client: Arc<SolanaClientPool>,
        processor_keypair: Arc<Keypair>,
        config: Config,
    ) -> Self {
        Self {
            blockchain_client,
            solana_client,
            processor_keypair,
            config,
        }
    }

    pub async fn run(&self) {
        let poll_interval = Duration::from_secs(self.config.blockchain.poll_interval_seconds);
        
        info!(
            poll_interval_seconds = self.config.blockchain.poll_interval_seconds,
            batch_size = self.config.blockchain.settlement_batch_size,
            "Settlement worker starting"
        );

        loop {
            debug!("Starting settlement batch processing cycle");
            
            if let Err(e) = self.process_batch().await {
                error!(error = %e, "Settlement batch processing failed");
            }

            sleep(poll_interval).await;
        }
    }

    async fn process_batch(&self) -> Result<()> {
        // Fetch pending settlements from blockchain API
        let games = self.blockchain_client
            .fetch_pending_settlements(self.config.blockchain.settlement_batch_size)
            .await
            .context("Failed to fetch pending settlements")?;

        if games.is_empty() {
            info!("No pending settlements");
            return Ok(());
        }

        info!(pending_count = games.len(), "Processing settlements");

        // Process each settlement
        for game in games {
            if let Err(e) = self.process_settlement(game).await {
                // Log error but continue with other settlements
                error!(error = %e, "Settlement processing failed");
            }
        }

        Ok(())
    }

    async fn process_settlement(&self, game: GameSettlementInfo) -> Result<()> {
        let tx_id = game.transaction_id;
        
        debug!(
            tx_id,
            player = %game.player_address,
            outcome = %game.outcome,
            payout = game.payout,
            "Processing settlement"
        );

        // Update status to SubmittedToSolana
        match self.blockchain_client
            .update_settlement_status(
                tx_id,
                "SubmittedToSolana",
                None,
                None,
                game.version,
            )
            .await
        {
            Ok(_) => {
                info!(tx_id, "Status updated to SubmittedToSolana");
            }
            Err(e) => {
                error!(tx_id, error = %e, "Failed to update status to SubmittedToSolana");
                return Err(e).context("Failed to update status to SubmittedToSolana");
            }
        }

        // Process on Solana
        let solana_tx_sig = match self.settle_on_solana(&game).await {
            Ok(sig) => sig,
            Err(e) => {
                let error_msg = format!("Solana settlement failed: {}", e);
                warn!(
                    tx_id,
                    error = %e,
                    "Solana settlement failed, updating status to SettlementFailed"
                );
                
                // Update status to SettlementFailed
                if let Err(update_err) = self.blockchain_client
                    .update_settlement_status(
                        tx_id,
                        "SettlementFailed",
                        None,
                        Some(error_msg),
                        game.version + 1,
                    )
                    .await
                {
                    error!(
                        tx_id,
                        solana_error = %e,
                        update_error = %update_err,
                        "Failed to update settlement status to SettlementFailed"
                    );
                }
                
                return Err(e);
            }
        };

        // Update status to SettlementComplete
        match self.blockchain_client
            .update_settlement_status(
                tx_id,
                "SettlementComplete",
                Some(solana_tx_sig.clone()),
                None,
                game.version + 1,
            )
            .await
        {
            Ok(_) => {
                info!(tx_id, solana_tx = %solana_tx_sig, "Status updated to SettlementComplete");
            }
            Err(e) => {
                error!(
                    tx_id,
                    solana_tx = %solana_tx_sig,
                    error = %e,
                    "CRITICAL: Solana settlement succeeded but blockchain status update failed"
                );
                return Err(e).context("Failed to update status to SettlementComplete");
            }
        }

        info!(
            tx_id,
            solana_tx = %solana_tx_sig,
            "Settlement completed"
        );

        Ok(())
    }

    async fn settle_on_solana(&self, game: &GameSettlementInfo) -> Result<String> {
        let bet_id = format!("bet-{}", game.transaction_id);
        
        // Determine if win or loss
        let is_win = game.outcome == "Win";

        if is_win {
            // Win: payout from casino vault
            self.process_payout(game, &bet_id).await
        } else {
            // Loss: spend from user's allowance
            self.process_spend(game, &bet_id).await
        }
    }

    async fn process_payout(&self, game: &GameSettlementInfo, bet_id: &str) -> Result<String> {
        use solana_sdk::{transaction::Transaction, system_program};
        use crate::solana_pda::{derive_casino_pda, derive_user_vault_pda};
        use crate::solana_instructions::build_payout_instruction;
        
        // Parse addresses
        let player_pubkey = game.player_address.parse()
            .context("Invalid player address")?;
        let vault_program_id = self.config.solana.vault_program_id.parse()?;

        // Derive PDAs
        let (casino_pda, _) = derive_casino_pda(&vault_program_id);
        let (user_vault_pda, _) = derive_user_vault_pda(&player_pubkey, &casino_pda, &vault_program_id);
        let (casino_vault, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"casino-vault", casino_pda.as_ref()],
            &vault_program_id,
        );
        let (vault_authority, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"vault-authority", casino_pda.as_ref()],
            &vault_program_id,
        );

        // Derive PDA for processed bet
        let (processed_bet_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"processed-bet", bet_id.as_bytes()],
            &vault_program_id,
        );

        // Build payout instruction
        let payout_ix = build_payout_instruction(
            &vault_program_id,
            &casino_pda,
            &casino_vault,
            &vault_authority,
            &user_vault_pda,
            &processed_bet_pda,
            &self.processor_keypair.pubkey(),
            game.payout,
            bet_id,
        );

        // Get recent blockhash and send
        let client = self.solana_client.get_client().await;
        let recent_blockhash = client.get_latest_blockhash()?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[payout_ix],
            Some(&self.processor_keypair.pubkey()),
            &[&*self.processor_keypair],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    async fn process_spend(&self, game: &GameSettlementInfo, bet_id: &str) -> Result<String> {
        use solana_sdk::transaction::Transaction;
        use crate::solana_pda::{derive_casino_pda, derive_user_vault_pda, derive_latest_allowance_pda_from_nonce_registry};
        use crate::solana_instructions::build_spend_from_allowance_instruction;
        
        // Parse addresses
        let player_pubkey = game.player_address.parse()
            .context("Invalid player address")?;
        let vault_program_id = self.config.solana.vault_program_id.parse()?;

        // Derive PDAs
        let (casino_pda, _) = derive_casino_pda(&vault_program_id);
        let (user_vault_pda, _) = derive_user_vault_pda(&player_pubkey, &casino_pda, &vault_program_id);
        let (casino_vault, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"casino-vault", casino_pda.as_ref()],
            &vault_program_id,
        );
        let (vault_authority, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"vault-authority", casino_pda.as_ref()],
            &vault_program_id,
        );

        // Get client for allowance lookup
        let client = self.solana_client.get_client().await;
        
        // Derive allowance PDA
        let allowance = derive_latest_allowance_pda_from_nonce_registry(
            &*client,
            &vault_program_id,
            &player_pubkey,
            &casino_pda,
        ).context("Failed to derive allowance PDA")?;

        // Derive PDA for processed bet
        let (processed_bet_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"processed-bet", bet_id.as_bytes()],
            &vault_program_id,
        );

        // Build spend instruction
        let spend_ix = build_spend_from_allowance_instruction(
            &vault_program_id,
            &user_vault_pda,
            &casino_pda,
            &allowance,
            &processed_bet_pda,
            &casino_vault,
            &vault_authority,
            None, // user_token_account
            None, // casino_token_account
            &self.processor_keypair.pubkey(),
            game.bet_amount,
            bet_id,
        );

        // Get recent blockhash and send
        let recent_blockhash = client.get_latest_blockhash()?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[spend_ix],
            Some(&self.processor_keypair.pubkey()),
            &[&*self.processor_keypair],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }
}
