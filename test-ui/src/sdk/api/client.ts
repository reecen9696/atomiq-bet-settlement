import type { AtomikConfig } from "../env";

export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface CoinflipRequest {
  choice: "heads" | "tails";
  amount: number;
  userPubkey: string;
  vaultPda: string;
  allowancePda?: string;
}

export interface CoinflipResult {
  gameId: string;
  outcome: "heads" | "tails";
  won: boolean;
  amount: number;
  timestamp: string;
}

export interface Settlement {
  settlementId: string;
  status: "pending" | "processing" | "completed" | "failed";
  gameId: string;
  amount: number;
  createdAt: string;
  processedAt?: string;
}

export interface RecentGame {
  gameId: string;
  gameType: string;
  outcome: string;
  amount: number;
  won: boolean;
  timestamp: string;
  playerPubkey: string;
}

export interface PaginatedGames {
  games: RecentGame[];
  cursor?: string;
  hasMore: boolean;
}

/**
 * Enhanced API client with retry logic and proper error handling
 */
export class AtomikApiClient {
  private config: AtomikConfig["api"];
  private baseHeaders: Record<string, string>;

  constructor(config: AtomikConfig) {
    this.config = config.api;
    this.baseHeaders = {
      "Content-Type": "application/json",
    };

    if (this.config.settlementApiKey) {
      this.baseHeaders["Authorization"] =
        `Bearer ${this.config.settlementApiKey}`;
    }
  }

  /**
   * Make a request with automatic retry logic
   */
  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
    attempt = 1,
  ): Promise<ApiResponse<T>> {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(
        () => controller.abort(),
        this.config.timeout,
      );

      const response = await fetch(`${this.config.baseUrl}${endpoint}`, {
        ...options,
        headers: { ...this.baseHeaders, ...options.headers },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      const data = await response.json();

      if (!response.ok) {
        throw new Error(
          data.error || data.message || `HTTP ${response.status}`,
        );
      }

      return {
        success: true,
        data: data.data || data,
        message: data.message,
      };
    } catch (error) {
      const isLastAttempt = attempt >= this.config.retryAttempts;

      if (!isLastAttempt && !(error as Error)?.name?.includes("AbortError")) {
        // Wait before retry (exponential backoff)
        await new Promise((resolve) =>
          setTimeout(resolve, Math.pow(2, attempt) * 1000),
        );
        return this.request<T>(endpoint, options, attempt + 1);
      }

      return {
        success: false,
        error: (error as Error)?.message || "Request failed",
      };
    }
  }

  /**
   * Play a coinflip game
   */
  async playCoinflip(
    request: CoinflipRequest,
  ): Promise<ApiResponse<CoinflipResult>> {
    return this.request<CoinflipResult>("/api/coinflip", {
      method: "POST",
      body: JSON.stringify(request),
    });
  }

  /**
   * Get game result by ID
   */
  async getGameResult(gameId: string): Promise<ApiResponse<CoinflipResult>> {
    return this.request<CoinflipResult>(`/api/games/${gameId}`);
  }

  /**
   * Get pending settlements for a user
   */
  async getPendingSettlements(
    userPubkey: string,
  ): Promise<ApiResponse<Settlement[]>> {
    return this.request<Settlement[]>(`/api/settlements/pending/${userPubkey}`);
  }

  /**
   * Get settlement details
   */
  async getSettlement(settlementId: string): Promise<ApiResponse<Settlement>> {
    return this.request<Settlement>(`/api/settlements/${settlementId}`);
  }

  /**
   * Get recent games with pagination
   */
  async getRecentGames(cursor?: string): Promise<ApiResponse<PaginatedGames>> {
    const params = cursor ? `?cursor=${cursor}` : "";
    return this.request<PaginatedGames>(`/api/games/recent${params}`);
  }

  /**
   * Health check
   */
  async healthCheck(): Promise<ApiResponse<{ status: string }>> {
    return this.request<{ status: string }>("/health");
  }

  /**
   * Get WebSocket URL for real-time updates
   */
  getWebSocketUrl(): string {
    return this.config.baseUrl.replace(/^http/, "ws");
  }
}

/**
 * Factory function to create an API client from config
 */
export function createApiClient(config: AtomikConfig): AtomikApiClient {
  return new AtomikApiClient(config);
}
