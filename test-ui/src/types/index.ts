export interface Token {
  symbol: string;
  mint_address?: string | null;
}

export type CoinChoice = "heads" | "tails";

export interface CoinFlipPlayRequest {
  player_id: string;
  choice: CoinChoice;
  token: Token;
  bet_amount: number;
  wallet_signature?: string | null;
  allowance_pda?: string | null;
}

export type GameResponse =
  | {
      status: "complete";
      game_id: string;
      result: GameResult;
    }
  | {
      status: "pending";
      game_id: string;
      message?: string | null;
    };

export interface GameResult {
  game_id: string;
  game_type: string;
  player: {
    player_id: string;
    wallet_signature?: string | null;
  };
  payment: {
    token: Token;
    bet_amount: number;
    payout_amount: number;
    settlement_tx_id?: string | null;
  };
  vrf: {
    vrf_output: string;
    vrf_proof: string;
    public_key: string;
    input_message: string;
  };
  outcome: string;
  timestamp: number;
  game_type_data: string;
  player_choice?: string;
  result_choice?: string;
  metadata?: unknown;
}

export interface SettlementGame {
  transaction_id: number;
  player_address: string;
  game_type: string;
  bet_amount: number;
  token: string;
  outcome: string;
  payout: number;
  vrf_proof: string;
  vrf_output: string;
  block_height: number;
  version: number;
}

export type SettlementStatus =
  | "PendingSettlement"
  | "SubmittedToSolana"
  | "SettlementComplete"
  | "SettlementFailed"
  | (string & {});

export interface SettlementGameDetail extends SettlementGame {
  block_hash: string;
  settlement_status: SettlementStatus;
  solana_tx_id?: string | null;
  settlement_error?: string | null;
  settlement_completed_at?: number | null;
}

export interface PendingSettlementsResponse {
  games: SettlementGame[];
  next_cursor?: string | null;
}

export interface RecentGameSummary {
  game_id: string;
  tx_id: number;
  processed?: boolean;
  settlement_status?: SettlementStatus;
  solana_tx_id?: string | null;
  settlement_error?: string | null;
  settlement_completed_at?: number | null;
  player_id: string;
  game_type: string;
  token: Token;
  bet_amount: number;
  player_choice: string;
  coin_result: string;
  outcome: string;
  payout: number;
  timestamp: number;
  block_height: number;
  block_hash: string;
}

export interface RecentGamesResponse {
  games: RecentGameSummary[];
  next_cursor?: string | null;
}

export interface VaultInfo {
  address: string;
  balance: number;
  allowance: number | null;
}
