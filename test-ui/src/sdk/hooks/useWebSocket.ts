import { useState, useEffect, useCallback, useRef } from "react";
import type {
  AtomikWebSocketManager,
  WebSocketConnection,
} from "../index";

export interface CasinoStats {
  totalGames: number;
  totalVolume: string;
  activeUsers: number;
  headsWins: number;
  tailsWins: number;
}

export interface RecentWin {
  gameId: string;
  outcome: "heads" | "tails";
  amount: number;
  playerPubkey: string;
  timestamp: string;
}

export interface BlockUpdate {
  slot: number;
  blockTime: number;
  blockhash: string;
}

export interface UseWebSocketState {
  // Connection status
  connected: boolean;
  connecting: boolean;

  // Live data
  casinoStats: CasinoStats | null;
  recentWins: RecentWin[];
  latestBlock: BlockUpdate | null;

  // Error state
  error: string | null;
}

export interface UseWebSocketActions {
  // Connection management
  connect: () => Promise<void>;
  disconnect: () => void;
  reconnect: () => Promise<void>;

  // State management
  clearError: () => void;
  reset: () => void;
}

export interface UseWebSocketResult
  extends UseWebSocketState, UseWebSocketActions {}

/**
 * React hook for managing WebSocket connections and live casino data
 */
export function useWebSocket(
  wsManager: AtomikWebSocketManager,
  autoConnect = true,
): UseWebSocketResult {
  const [state, setState] = useState<UseWebSocketState>({
    connected: false,
    connecting: false,
    casinoStats: null,
    recentWins: [],
    latestBlock: null,
    error: null,
  });

  const connectionsRef = useRef<{
    stats: WebSocketConnection | null;
    wins: WebSocketConnection | null;
    blocks: WebSocketConnection | null;
  }>({ stats: null, wins: null, blocks: null });

  const unsubscribeFnsRef = useRef<(() => void)[]>([]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, []);

  // Auto-connect if enabled
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
  }, [autoConnect]);

  const connect = useCallback(async () => {
    setState((prev) => ({ ...prev, connecting: true, error: null }));

    try {
      // Connect to all casino streams
      const connections = await wsManager.connectToCasinoStreams();
      connectionsRef.current = connections;

      // Clear previous subscriptions
      unsubscribeFnsRef.current.forEach((fn) => fn());
      unsubscribeFnsRef.current = [];

      // Subscribe to casino stats
      const unsubStats = connections.stats.subscribe<CasinoStats>(
        "casino-stats",
        (data) => {
          setState((prev) => ({ ...prev, casinoStats: data }));
        },
      );

      // Subscribe to recent wins
      const unsubWins = connections.wins.subscribe<RecentWin>(
        "recent-win",
        (data) => {
          setState((prev) => ({
            ...prev,
            recentWins: [data, ...prev.recentWins.slice(0, 9)], // Keep last 10 wins
          }));
        },
      );

      // Subscribe to block updates
      const unsubBlocks = connections.blocks.subscribe<BlockUpdate>(
        "block-update",
        (data) => {
          setState((prev) => ({ ...prev, latestBlock: data }));
        },
      );

      // Store unsubscribe functions
      unsubscribeFnsRef.current = [unsubStats, unsubWins, unsubBlocks];

      // Set up connection event handlers
      const statsConnectUnsub = connections.stats.onConnect(() => {
        setState((prev) => ({ ...prev, connected: true, connecting: false }));
      });

      const statsDisconnectUnsub = connections.stats.onDisconnect(() => {
        setState((prev) => ({ ...prev, connected: false }));
      });

      const statsErrorUnsub = connections.stats.onError(() => {
        setState((prev) => ({
          ...prev,
          error: "WebSocket connection error",
          connecting: false,
        }));
      });

      unsubscribeFnsRef.current.push(
        statsConnectUnsub,
        statsDisconnectUnsub,
        statsErrorUnsub,
      );

      setState((prev) => ({ ...prev, connected: true, connecting: false }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        error: (error as Error).message || "Failed to connect to WebSocket",
        connecting: false,
        connected: false,
      }));
    }
  }, [wsManager]);

  const disconnect = useCallback(() => {
    // Unsubscribe from all events
    unsubscribeFnsRef.current.forEach((fn) => fn());
    unsubscribeFnsRef.current = [];

    // Disconnect all connections
    wsManager.disconnectAll();
    connectionsRef.current = { stats: null, wins: null, blocks: null };

    setState((prev) => ({ ...prev, connected: false, connecting: false }));
  }, [wsManager]);

  const reconnect = useCallback(async () => {
    disconnect();
    await new Promise((resolve) => setTimeout(resolve, 1000)); // Wait 1 second
    await connect();
  }, [connect, disconnect]);

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  const reset = useCallback(() => {
    disconnect();
    setState({
      connected: false,
      connecting: false,
      casinoStats: null,
      recentWins: [],
      latestBlock: null,
      error: null,
    });
  }, [disconnect]);

  return {
    ...state,
    connect,
    disconnect,
    reconnect,
    clearError,
    reset,
  };
}

/**
 * Hook for managing a single WebSocket connection type
 */
export function useWebSocketConnection(
  wsManager: AtomikWebSocketManager,
  connectionName: string,
  url?: string,
) {
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connectionRef = useRef<WebSocketConnection | null>(null);
  const unsubscribeFnsRef = useRef<(() => void)[]>([]);

  useEffect(() => {
    return () => {
      if (connectionRef.current) {
        connectionRef.current.disconnect();
      }
      unsubscribeFnsRef.current.forEach((fn) => fn());
    };
  }, []);

  const connect = useCallback(async () => {
    setConnecting(true);
    setError(null);

    try {
      const connection = wsManager.getConnection(connectionName, url);
      connectionRef.current = connection;

      // Set up event handlers
      const connectUnsub = connection.onConnect(() => {
        setConnected(true);
        setConnecting(false);
      });

      const disconnectUnsub = connection.onDisconnect(() => {
        setConnected(false);
      });

      const errorUnsub = connection.onError(() => {
        setError("Connection error");
        setConnecting(false);
      });

      unsubscribeFnsRef.current = [connectUnsub, disconnectUnsub, errorUnsub];

      await connection.connect();
    } catch (err) {
      setError((err as Error).message || "Failed to connect");
      setConnecting(false);
    }
  }, [wsManager, connectionName, url]);

  const disconnect = useCallback(() => {
    if (connectionRef.current) {
      connectionRef.current.disconnect();
      connectionRef.current = null;
    }

    unsubscribeFnsRef.current.forEach((fn) => fn());
    unsubscribeFnsRef.current = [];

    setConnected(false);
    setConnecting(false);
  }, []);

  const subscribe = useCallback(
    <T>(messageType: string, handler: (data: T) => void) => {
      if (!connectionRef.current) {
        throw new Error("Connection not established");
      }

      return connectionRef.current.subscribe(messageType, handler);
    },
    [],
  );

  return {
    connected,
    connecting,
    error,
    connect,
    disconnect,
    subscribe,
    connection: connectionRef.current,
  };
}
