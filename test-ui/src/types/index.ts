export interface Bet {
  bet_id: string;
  created_at: string;
  user_wallet: string;
  vault_address: string;
  casino_id: string | null;
  game_type: string;
  stake_amount: number;
  stake_token: string;
  choice: string;
  status: BetStatus;
  external_batch_id: string | null;
  solana_tx_id: string | null;
  retry_count: number;
  processor_id: string | null;
  last_error_code: string | null;
  last_error_message: string | null;
  payout_amount: number | null;
  won: boolean | null;
}

export type BetStatus =
  // Backend (Redis POC) returns lowercase snake_case
  | 'pending'
  | 'batched'
  | 'submitted_to_solana'
  | 'confirmed_on_solana'
  | 'completed'
  | 'failed_retryable'
  | 'failed_manual_review'
  // Legacy/UI-friendly variants
  | 'Pending'
  | 'Batched'
  | 'SubmittedToSolana'
  | 'ConfirmedOnSolana'
  | 'Completed'
  | 'FailedRetryable'
  | 'FailedManualReview'
  // Fallback for unknown/new statuses
  | (string & {});

export interface CreateBetRequest {
  user_wallet?: string;
  vault_address?: string;
  allowance_pda?: string;
  stake_amount: number;
  stake_token: string;
  choice: string;
}

export interface CreateBetResponse {
  bet: Bet;
}

export interface VaultInfo {
  address: string;
  balance: number;
  allowance: number | null;
}