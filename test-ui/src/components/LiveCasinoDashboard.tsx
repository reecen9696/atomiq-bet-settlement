import { useState, useEffect, useRef } from "react";

interface CasinoStats {
  total_wagered: number;
  gross_rtp: number;
  bet_count: number;
  bankroll: number;
  wins_24h: number;
  wagered_24h: number;
}

interface CasinoWin {
  game_type: string;
  wallet: string;
  amount_won: number;
  currency: string;
  timestamp: number;
  tx_id: string;
  block_height: number;
}

interface Block {
  height: number;
  hash: string;
  tx_count: number;
  timestamp: number;
}

export function LiveCasinoDashboard() {
  const [stats, setStats] = useState<CasinoStats | null>(null);
  const [recentWins, setRecentWins] = useState<CasinoWin[]>([]);
  const [recentBlocks, setRecentBlocks] = useState<Block[]>([]);
  const [wsConnected, setWsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const maxReconnectAttempts = 10;

  const apiUrl = import.meta.env.VITE_API_BASE_URL || "http://localhost:8080";

  // Fetch initial data (blocks and recent wins)
  useEffect(() => {
    const fetchInitialData = async () => {
      try {
        setLoading(true);

        // Fetch recent blocks
        const blocksResponse = await fetch(`${apiUrl}/blocks?limit=5`);
        if (blocksResponse.ok) {
          const blocksData = await blocksResponse.json();
          if (blocksData.blocks && Array.isArray(blocksData.blocks)) {
            setRecentBlocks(
              blocksData.blocks.map((block: any) => ({
                height: block.height,
                hash: block.hash,
                tx_count: block.tx_count || 0,
                timestamp: block.time
                  ? new Date(block.time).getTime()
                  : Date.now(),
              })),
            );
          }
        }

        // Fetch recent wins (fetch more to ensure we get 5 wins after filtering)
        const winsResponse = await fetch(`${apiUrl}/api/games/recent?limit=20`);
        if (winsResponse.ok) {
          const winsData = await winsResponse.json();
          if (winsData.games && Array.isArray(winsData.games)) {
            // Filter only wins and map to CasinoWin format
            const wins = winsData.games
              .filter((game: any) => game.outcome === "win")
              .slice(0, 5)
              .map((game: any) => ({
                game_type: game.game_type || "coinflip",
                wallet: game.player_id || "unknown",
                amount_won: (game.payout || 0) / 1_000_000_000,
                currency: game.token?.symbol || "SOL",
                timestamp: game.timestamp,
                tx_id: game.tx_id.toString(),
                block_height: game.block_height || 0,
              }));
            setRecentWins(wins);
          }
        }

        setError(null);
      } catch (error) {
        console.error("Failed to fetch initial data:", error);
        setError("Failed to load initial data");
      } finally {
        setLoading(false);
      }
    };

    fetchInitialData();
  }, [apiUrl]);

  // Fetch initial stats
  useEffect(() => {
    const fetchStats = async () => {
      try {
        const response = await fetch(`${apiUrl}/api/casino/stats`);
        if (response.ok) {
          const data = await response.json();
          setStats(data);
          setError(null);
        } else {
          console.warn(`Stats API returned ${response.status}`);
        }
      } catch (error) {
        console.warn("Stats fetch failed (will retry):", error);
      }
    };

    const initialTimeout = setTimeout(fetchStats, 2000);
    const interval = setInterval(fetchStats, 10000);

    return () => {
      clearTimeout(initialTimeout);
      clearInterval(interval);
    };
  }, [apiUrl]);

  // WebSocket connection
  useEffect(() => {
    const wsUrl = apiUrl
      .replace("http://", "ws://")
      .replace("https://", "wss://");

    const connectWebSocket = () => {
      if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
        setError("Failed to connect to live updates after multiple attempts");
        return;
      }

      try {
        const ws = new WebSocket(`${wsUrl}/ws?casino=true&blocks=true`);
        wsRef.current = ws;

        ws.onopen = () => {
          console.log("✅ WebSocket connected");
          setWsConnected(true);
          setError(null);
          reconnectAttemptsRef.current = 0;
        };

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);

            if (data.type === "casino_win") {
              setRecentWins((prev) => [data, ...prev].slice(0, 10));
            } else if (data.type === "casino_stats") {
              setStats(data);
            } else if (data.type === "new_block") {
              setRecentBlocks((prev) =>
                [
                  {
                    height: data.height,
                    hash: data.hash,
                    tx_count: data.tx_count,
                    timestamp: data.timestamp,
                  },
                  ...prev,
                ].slice(0, 10),
              );
            }
          } catch (error) {
            console.error("Error parsing WebSocket message:", error);
          }
        };

        ws.onerror = (error) => {
          console.error("WebSocket error:", error);
          setWsConnected(false);
        };

        ws.onclose = () => {
          console.log("❌ WebSocket disconnected");
          setWsConnected(false);

          const delay = Math.min(
            1000 * Math.pow(2, reconnectAttemptsRef.current),
            30000,
          );
          reconnectAttemptsRef.current++;

          console.log(
            `Reconnecting in ${delay / 1000}s (attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts})`,
          );

          reconnectTimeoutRef.current = setTimeout(connectWebSocket, delay);
        };
      } catch (error) {
        console.error("Failed to create WebSocket:", error);
        const delay = Math.min(
          1000 * Math.pow(2, reconnectAttemptsRef.current),
          30000,
        );
        reconnectAttemptsRef.current++;
        reconnectTimeoutRef.current = setTimeout(connectWebSocket, delay);
      }
    };

    const initialTimeout = setTimeout(connectWebSocket, 1000);

    return () => {
      clearTimeout(initialTimeout);
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  const formatAddress = (address: string) => {
    if (address.length <= 12) return address;
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleTimeString();
  };

  const formatHash = (hash: string) => {
    if (hash.length <= 16) return hash;
    return `${hash.slice(0, 8)}...${hash.slice(-8)}`;
  };

  return (
    <div className="space-y-6">
      {/* Connection Status */}
      <div className="flex items-center gap-2 text-sm">
        <div
          className={`w-3 h-3 rounded-full ${wsConnected ? "bg-green-500 animate-pulse" : "bg-red-500"}`}
        />
        <span className="text-gray-600">
          {wsConnected ? "Live Updates Active" : "Connecting..."}
        </span>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4">
          <p className="text-sm text-yellow-700">{error}</p>
          <button
            onClick={() => {
              reconnectAttemptsRef.current = 0;
              setError(null);
              window.location.reload();
            }}
            className="mt-2 text-sm text-yellow-900 underline hover:no-underline"
          >
            Retry Connection
          </button>
        </div>
      )}

      {/* Casino Statistics */}
      <div className="bg-white rounded-lg shadow-lg border-l-4 border-purple-500 p-6">
        <h2 className="text-2xl font-bold text-gray-800 mb-4 flex items-center gap-2">
          📊 Casino Statistics
        </h2>
        {stats ? (
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            <div className="bg-gradient-to-br from-purple-50 to-purple-100 p-4 rounded-lg">
              <div className="text-sm text-purple-600 font-medium">
                Total Wagered
              </div>
              <div className="text-2xl font-bold text-purple-900">
                {stats.total_wagered.toFixed(4)} SOL
              </div>
            </div>
            <div className="bg-gradient-to-br from-blue-50 to-blue-100 p-4 rounded-lg">
              <div className="text-sm text-blue-600 font-medium">Gross RTP</div>
              <div className="text-2xl font-bold text-blue-900">
                {stats.gross_rtp.toFixed(2)}%
              </div>
            </div>
            <div className="bg-gradient-to-br from-green-50 to-green-100 p-4 rounded-lg">
              <div className="text-sm text-green-600 font-medium">
                Total Bets
              </div>
              <div className="text-2xl font-bold text-green-900">
                {stats.bet_count.toLocaleString()}
              </div>
            </div>
            <div className="bg-gradient-to-br from-orange-50 to-orange-100 p-4 rounded-lg">
              <div className="text-sm text-orange-600 font-medium">
                Wins (24h)
              </div>
              <div className="text-2xl font-bold text-orange-900">
                {stats.wins_24h}
              </div>
            </div>
            <div className="bg-gradient-to-br from-pink-50 to-pink-100 p-4 rounded-lg">
              <div className="text-sm text-pink-600 font-medium">
                Wagered (24h)
              </div>
              <div className="text-2xl font-bold text-pink-900">
                {stats.wagered_24h.toFixed(4)} SOL
              </div>
            </div>
            <div className="bg-gradient-to-br from-indigo-50 to-indigo-100 p-4 rounded-lg">
              <div className="text-sm text-indigo-600 font-medium">
                Bankroll
              </div>
              <div className="text-2xl font-bold text-indigo-900">
                {stats.bankroll.toFixed(4)} SOL
              </div>
            </div>
          </div>
        ) : (
          <div className="text-gray-500 text-center py-8">
            Loading statistics...
          </div>
        )}
      </div>

      {/* Recent Wins */}
      <div className="bg-white rounded-lg shadow-lg border-l-4 border-green-500 p-6">
        {loading ? (
          <div className="text-gray-500 text-center py-8">
            Loading recent wins...
          </div>
        ) : recentWins.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">
                    Game
                  </th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">
                    Player
                  </th>
                  <th className="px-4 py-2 text-right text-gray-600 font-medium">
                    Amount
                  </th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">
                    Block
                  </th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">
                    Time
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200">
                {recentWins.map((win, idx) => (
                  <tr
                    key={`${win.tx_id}-${idx}`}
                    className="hover:bg-gray-50 animate-fade-in"
                  >
                    <td className="px-4 py-3 font-medium text-gray-900">
                      {win.game_type}
                    </td>
                    <td className="px-4 py-3 font-mono text-sm text-gray-600">
                      {formatAddress(win.wallet)}
                    </td>
                    <td className="px-4 py-3 text-right font-bold text-green-600">
                      +{win.amount_won.toFixed(4)} {win.currency}
                    </td>
                    <td className="px-4 py-3 text-gray-600">
                      #{win.block_height}
                    </td>
                    <td className="px-4 py-3 text-gray-500 text-xs">
                      {formatTimestamp(win.timestamp)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="text-gray-500 text-center py-8">
            {wsConnected ? "Waiting for wins..." : "Connecting to live feed..."}
          </div>
        )}
      </div>

      {/* Recent Blocks */}
      <div className="bg-white rounded-lg shadow-lg border-l-4 border-blue-500 p-6">
        <h2 className="text-2xl font-bold text-gray-800 mb-4 flex items-center gap-2">
          📦 Latest Blocks
        </h2>
        {loading ? (
          <div className="text-gray-500 text-center py-8">
            Loading recent blocks...
          </div>
        ) : recentBlocks.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">
                    Height
                  </th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">
                    Hash
                  </th>
                  <th className="px-4 py-2 text-right text-gray-600 font-medium">
                    Transactions
                  </th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">
                    Time
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200">
                {recentBlocks.map((block) => (
                  <tr
                    key={block.height}
                    className="hover:bg-gray-50 animate-fade-in"
                  >
                    <td className="px-4 py-3 font-bold text-blue-600">
                      #{block.height}
                    </td>
                    <td className="px-4 py-3 font-mono text-xs text-gray-600">
                      {formatHash(block.hash)}
                    </td>
                    <td className="px-4 py-3 text-right text-gray-900 font-medium">
                      {block.tx_count}
                    </td>
                    <td className="px-4 py-3 text-gray-500 text-xs">
                      {formatTimestamp(block.timestamp)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="text-gray-500 text-center py-8">
            {wsConnected
              ? "Waiting for blocks..."
              : "Connecting to blockchain..."}
          </div>
        )}
      </div>
    </div>
  );
}
