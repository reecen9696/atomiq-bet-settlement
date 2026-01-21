import type {
  CoinFlipPlayRequest,
  GameResponse,
  PendingSettlementsResponse,
  RecentGamesResponse,
  SettlementGameDetail,
} from "../types";

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL || "http://localhost:3001";
const SETTLEMENT_API_KEY = import.meta.env.VITE_SETTLEMENT_API_KEY;

export class ApiService {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async parseError(response: Response): Promise<string> {
    try {
      const data: unknown = await response.json();
      if (data && typeof data === "object") {
        const maybeMessage = (data as any).error || (data as any).message;
        if (typeof maybeMessage === "string" && maybeMessage.length > 0)
          return maybeMessage;
      }
    } catch {
      // ignore
    }

    try {
      const text = await response.text();
      if (text.trim().length > 0) return text;
    } catch {
      // ignore
    }

    return `Request failed (${response.status})`;
  }

  async playCoinflip(request: CoinFlipPlayRequest): Promise<GameResponse> {
    const response = await fetch(`${this.baseUrl}/api/coinflip/play`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(await this.parseError(response));
    }

    return response.json();
  }

  async getPendingSettlements(params?: {
    limit?: number;
    cursor?: string;
  }): Promise<PendingSettlementsResponse> {
    const url = new URL(`${this.baseUrl}/api/settlement/pending`);
    url.searchParams.set("limit", String(params?.limit ?? 50));
    if (params?.cursor) url.searchParams.set("cursor", params.cursor);

    const response = await fetch(url.toString(), {
      headers: {
        ...(SETTLEMENT_API_KEY ? { "X-API-Key": SETTLEMENT_API_KEY } : {}),
      },
    });

    if (!response.ok) {
      throw new Error(await this.parseError(response));
    }

    return response.json();
  }

  async getGameResult(gameId: string): Promise<GameResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/game/${encodeURIComponent(gameId)}`,
    );
    if (!response.ok) throw new Error(await this.parseError(response));
    return response.json();
  }

  async getSettlementGame(txId: number): Promise<SettlementGameDetail> {
    const response = await fetch(
      `${this.baseUrl}/api/settlement/games/${txId}`,
      {
        headers: {
          ...(SETTLEMENT_API_KEY ? { "X-API-Key": SETTLEMENT_API_KEY } : {}),
        },
      },
    );
    if (!response.ok) throw new Error(await this.parseError(response));
    return response.json();
  }

  async getRecentGames(params?: {
    limit?: number;
    cursor?: string;
  }): Promise<RecentGamesResponse> {
    const url = new URL(`${this.baseUrl}/api/games/recent`);
    url.searchParams.set("limit", String(params?.limit ?? 50));
    if (params?.cursor) url.searchParams.set("cursor", params.cursor);

    const response = await fetch(url.toString());
    if (!response.ok) throw new Error(await this.parseError(response));
    return response.json();
  }

  async healthCheck(): Promise<{ status: string; timestamp: string }> {
    const response = await fetch(`${this.baseUrl}/health`);

    if (!response.ok) {
      throw new Error("Health check failed");
    }

    return response.json();
  }
}

export const apiService = new ApiService();
