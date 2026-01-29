// Export all SDK hooks for easy importing
export {
  useVault,
  type UseVaultState,
  type UseVaultActions,
  type UseVaultResult,
} from './useVault';

export {
  useAllowance,
  useAllowanceForSpender,
  type UseAllowanceState,
  type UseAllowanceActions,
  type UseAllowanceResult,
} from './useAllowance';

export {
  useBetting,
  useGameResult,
  type UseBettingState,
  type UseBettingActions,
  type UseBettingResult,
} from './useBetting';

export {
  useWebSocket,
  useWebSocketConnection,
  type UseWebSocketState,
  type UseWebSocketActions,
  type UseWebSocketResult,
  type CasinoStats,
  type RecentWin,
  type BlockUpdate,
} from './useWebSocket';

/**
 * Combined hook that provides all Atomik SDK functionality
 * Use this for full-featured applications that need all services
 */
import { useMemo } from 'react';
import type { AtomikConfig } from '../index';
import { createAtomikSDK } from '../index';
import { useVault } from './useVault';
import { useAllowance } from './useAllowance';
import { useBetting } from './useBetting';
import { useWebSocket } from './useWebSocket';

export interface UseAtomikSDKOptions {
  config?: Partial<AtomikConfig>;
  userPublicKey?: string | null | undefined;
  sendTransaction?: Function;
  signTransaction?: Function;
  autoConnectWebSocket?: boolean;
}

export function useAtomikSDK(options: UseAtomikSDKOptions = {}) {
  const {
    config = {},
    userPublicKey,
    sendTransaction,
    signTransaction,
    autoConnectWebSocket = true,
  } = options;

  // Create SDK instance (memoized to prevent recreation)
  const sdk = useMemo(() => createAtomikSDK(config), [config]);

  // Individual service hooks
  const vault = useVault(userPublicKey ?? null, sdk.vault, sendTransaction, signTransaction);
  const allowance = useAllowance(userPublicKey ?? null, sdk.allowance, sendTransaction, signTransaction);
  const betting = useBetting(userPublicKey ?? null, sdk.betting);
  const websocket = useWebSocket(sdk.websocket, autoConnectWebSocket);

  return {
    // SDK services
    sdk,
    
    // Individual hooks with state management
    vault,
    allowance,
    betting,
    websocket,
    
    // Quick access to raw services (for advanced use cases)
    services: {
      api: sdk.api,
      vaultService: sdk.vault,
      allowanceService: sdk.allowance,
      bettingService: sdk.betting,
      websocketManager: sdk.websocket,
    },
  };
}