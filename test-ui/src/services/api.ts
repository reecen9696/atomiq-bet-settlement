import type { Bet, CreateBetRequest, CreateBetResponse } from '../types';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001';

export class ApiService {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  async createBet(request: CreateBetRequest): Promise<CreateBetResponse> {
    const response = await fetch(`${this.baseUrl}/api/bets`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to create bet');
    }

    return response.json();
  }

  async getPendingBets(): Promise<Bet[]> {
    // NOTE: This endpoint is intended for the processor (it also claims bets).
    // Keep it only for debugging and normalize the response shape.
    const response = await fetch(`${this.baseUrl}/api/external/bets/pending?limit=50&processor_id=test-ui`);

    if (!response.ok) {
      throw new Error('Failed to fetch pending bets');
    }

    const data: unknown = await response.json();
    if (Array.isArray(data)) return data as Bet[];
    if (data && typeof data === 'object' && Array.isArray((data as any).bets)) return (data as any).bets as Bet[];
    return [];
  }

  async listUserBets(userWallet: string): Promise<Bet[]> {
    const url = new URL(`${this.baseUrl}/api/bets`);
    url.searchParams.set('user_wallet', userWallet);
    url.searchParams.set('limit', '50');

    const response = await fetch(url.toString());
    if (!response.ok) {
      throw new Error('Failed to fetch user bets');
    }

    const data: unknown = await response.json();
    return Array.isArray(data) ? (data as Bet[]) : [];
  }

  async getAllBets(): Promise<Bet[]> {
    // Deprecated: in this POC, use listUserBets(userWallet)
    return this.getPendingBets();
  }

  async healthCheck(): Promise<{ status: string; timestamp: string }> {
    const response = await fetch(`${this.baseUrl}/health`);
    
    if (!response.ok) {
      throw new Error('Health check failed');
    }

    return response.json();
  }
}

export const apiService = new ApiService();