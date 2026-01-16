import { z } from 'zod';

export enum BetStatus {
  Pending = 'pending',
  Batched = 'batched',
  SubmittedToSolana = 'submitted_to_solana',
  ConfirmedOnSolana = 'confirmed_on_solana',
  Completed = 'completed',
  FailedRetryable = 'failed_retryable',
  FailedManualReview = 'failed_manual_review',
}

export const BetSchema = z.object({
  bet_id: z.string().uuid(),
  created_at: z.string().datetime(),
  user_wallet: z.string(),
  vault_address: z.string(),
  casino_id: z.string().optional(),
  game_type: z.string(),
  stake_amount: z.number().int().positive(),
  stake_token: z.string(),
  choice: z.string(),
  status: z.nativeEnum(BetStatus),
  external_batch_id: z.string().uuid().optional(),
  solana_tx_id: z.string().optional(),
  retry_count: z.number().int(),
  processor_id: z.string().optional(),
  last_error_code: z.string().optional(),
  last_error_message: z.string().optional(),
  payout_amount: z.number().int().optional(),
  won: z.boolean().optional(),
});

export type Bet = z.infer<typeof BetSchema>;

export const CreateBetRequestSchema = z.object({
  stake_amount: z.number().int().positive(),
  stake_token: z.string(),
  choice: z.enum(['heads', 'tails']),
});

export type CreateBetRequest = z.infer<typeof CreateBetRequestSchema>;

export const VaultBalanceSchema = z.object({
  sol_balance: z.number(),
  usdc_balance: z.number(),
});

export type VaultBalance = z.infer<typeof VaultBalanceSchema>;

export const AllowanceSchema = z.object({
  amount: z.number(),
  spent: z.number(),
  remaining: z.number(),
  expires_at: z.string().datetime(),
  token_mint: z.string(),
  revoked: z.boolean(),
});

export type Allowance = z.infer<typeof AllowanceSchema>;

export const HealthStatusSchema = z.object({
  status: z.enum(['healthy', 'degraded', 'unhealthy']),
  timestamp: z.string(),
  components: z
    .object({
      database: z.string(),
      redis: z.string(),
    })
    .optional(),
});

export type HealthStatus = z.infer<typeof HealthStatusSchema>;
