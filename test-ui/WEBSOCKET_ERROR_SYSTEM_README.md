# WebSocket Error Message System - Implementation Guide

## Overview

The test-ui implements a comprehensive **real-time error notification system** using WebSockets to provide immediate feedback on transaction settlements, connection issues, and system failures.

## Architecture Components

### 1. WebSocket Management Layer

**Core Manager**: `src/sdk/websocket/manager.ts`

```typescript
// AtomikWebSocketManager - Central hub for multiple connections
class AtomikWebSocketManager {
  // Manages connections for stats, recent_wins, block_updates
  // Auto-reconnection with exponential backoff
  // Connection state tracking and error recovery
}

// WebSocketConnection - Individual connection wrapper
class WebSocketConnection {
  // Auto-reconnection logic
  // Message routing and error handling
  // Connection state management
}
```

**React Hook Interface**: `src/sdk/hooks/useWebSocket.ts`

```typescript
// Main hook for WebSocket connections and live data
const useWebSocket = () => {
  // Manages connection states for all WebSocket types
  // Provides error states and connection status
  // Handles automatic reconnection attempts
};

// Hook for individual connection management
const useWebSocketConnection = (url: string, options) => {
  // Single connection lifecycle management
  // Error handling and retry logic
  // Message subscription system
};
```

### 2. Real-time Message Types

The system handles three main categories of real-time data:

1. **Casino Stats**: Live casino statistics and metrics
2. **Recent Wins**: Real-time win notifications and payouts
3. **Block Updates**: Blockchain block information and confirmations

### 3. Settlement Failure Notification System

**Location**: `src/components/LiveCasinoDashboard.tsx` (lines 156-177)

```typescript
// Real-time settlement failure handling
else if (data.type === "settlement_failed") {
  const failureMessage = data.is_permanent
    ? `Settlement failed permanently: ${data.error_message}`
    : `Settlement failed (retry ${data.retry_count}): ${data.error_message}`;

  setNotifications((prev) => [
    {
      id: Date.now(),
      type: "settlement_failed",
      message: failureMessage,
      details: `Transaction ID: ${data.transaction_id} | Bet: ${data.bet_amount} SOL`,
      timestamp: Date.now(),
    },
    ...prev,
  ].slice(0, 50)); // Keep last 50 notifications
}
```

**Key Features**:

- **Immediate Error Feedback**: Real-time settlement failure notifications
- **Retry Information**: Shows current retry attempt count
- **Transaction Context**: Includes transaction ID and bet amount
- **Permanent vs Temporary**: Distinguishes between permanent failures and retries
- **Notification History**: Maintains last 50 notifications for user review

## Error Handling Architecture

### 1. Connection-Level Error Handling

**Auto-Reconnection Logic** (`manager.ts`):

```typescript
// Exponential backoff reconnection
const retryDelay = Math.min(1000 * Math.pow(2, retryCount), 30000);

// Error event handlers
websocket.onerror = (error) => {
  console.error("WebSocket error:", error);
  this.handleConnectionError();
};

websocket.onclose = (event) => {
  if (!this.isDestroyed && event.code !== 1000) {
    this.scheduleReconnection();
  }
};
```

### 2. Hook-Level Error Management

**useWebSocket Error States** (`useWebSocket.ts`):

```typescript
// Connection error handling
const statsErrorUnsub = connections.stats.onError(() => {
  setState((prev) => ({
    ...prev,
    error: "WebSocket connection error",
    connecting: false,
  }));
});

// Provides error state to components
return {
  isConnected: state.isConnected,
  error: state.error,
  connecting: state.connecting,
  // ... other state
};
```

### 3. UI Error Display System

**Notification Panel** (`LiveCasinoDashboard.tsx` lines 430-479):

```typescript
// Error notification display
{notifications.length > 0 && (
  <div className="bg-red-50 border border-red-200 rounded-lg p-4">
    <div className="flex items-center justify-between mb-2">
      <h3 className="text-sm font-semibold text-red-800 flex items-center">
        <AlertTriangle className="w-4 h-4 mr-1" />
        Settlement Notifications ({notifications.length})
      </h3>
      <button
        onClick={() => setNotifications([])}
        className="text-red-600 hover:text-red-800 text-xs"
      >
        Clear All
      </button>
    </div>

    <div className="max-h-32 overflow-y-auto space-y-1">
      {notifications.map((notification) => (
        <div
          key={notification.id}
          className="text-xs text-red-700 bg-red-100 p-2 rounded flex items-start justify-between"
        >
          <div className="flex-1">
            <div className="font-medium">{notification.message}</div>
            {notification.details && (
              <div className="text-red-600 mt-1">{notification.details}</div>
            )}
            <div className="text-red-500 mt-1">
              {new Date(notification.timestamp).toLocaleTimeString()}
            </div>
          </div>
          <button
            onClick={() => dismissNotification(notification.id)}
            className="ml-2 text-red-600 hover:text-red-800"
          >
            ×
          </button>
        </div>
      ))}
    </div>
  </div>
)}
```

## Error Display Patterns

### 1. Inline Component Errors

**BettingInterface Error Display** (`BettingInterface.tsx`):

```typescript
// Local error state management
const [error, setError] = useState<string>("");

// Error display in UI
{error && (
  <div className="p-3 bg-red-50 border border-red-200 rounded-md flex items-center">
    <AlertCircle className="w-5 h-5 text-red-500 mr-2" />
    <span className="text-sm text-red-700">{error}</span>
  </div>
)}
```

### 2. Connection Status Indicators

**WebSocket Status Display**:

```typescript
// Connection state visualization
{connecting && <div className="text-yellow-600">Connecting...</div>}
{!isConnected && !connecting && (
  <div className="text-red-600">Connection failed</div>
)}
{isConnected && <div className="text-green-600">Connected</div>}
```

### 3. API Error Handling

**Standardized Error Parsing** (`services/api.ts`):

```typescript
// Extract error messages from API responses
function getErrorMessage(error: any): string {
  if (typeof error === "string") return error;
  if (error?.message) return error.message;
  if (error?.error) return error.error;
  if (error?.details) return error.details;
  return "An unexpected error occurred";
}
```

## Message Flow Architecture

```
1. Backend Settlement Processor
   ↓ (WebSocket Message)
2. WebSocket Manager (manager.ts)
   ↓ (Message Routing)
3. useWebSocket Hook (useWebSocket.ts)
   ↓ (State Management)
4. LiveCasinoDashboard Component
   ↓ (Notification Creation)
5. UI Error Display Panel
   ↓ (User Interaction)
6. Notification Dismissal/Management
```

## Key Implementation Features

### 1. Real-time Error Communication

- **Immediate Feedback**: Settlement failures show up instantly
- **Context Rich**: Includes transaction IDs, bet amounts, error details
- **User Friendly**: Clear error messages and actionable information

### 2. Robust Connection Management

- **Auto-Reconnection**: Handles network interruptions gracefully
- **Exponential Backoff**: Prevents connection spam during outages
- **State Tracking**: Provides clear connection status to users

### 3. Error Categorization

- **Settlement Failures**: Transaction processing errors
- **Connection Errors**: WebSocket connectivity issues
- **API Errors**: Backend service failures
- **Validation Errors**: User input and state validation

### 4. User Experience Features

- **Notification History**: Keep last 50 error notifications
- **Individual Dismissal**: Users can dismiss specific notifications
- **Bulk Clear**: Clear all notifications at once
- **Timestamp Display**: Show when errors occurred
- **Visual Indicators**: Color-coded error states and icons

## Files in the WebSocket Error System

1. **`src/sdk/websocket/manager.ts`** - Core WebSocket management and reconnection
2. **`src/sdk/hooks/useWebSocket.ts`** - React hooks for WebSocket integration
3. **`src/components/LiveCasinoDashboard.tsx`** - Main error notification display
4. **`src/services/api.ts`** - API error standardization and parsing
5. **`src/components/BettingInterface.tsx`** - Inline error display patterns

## Benefits

1. **Real-time Awareness**: Users immediately know about transaction issues
2. **Better Debugging**: Detailed error context helps identify problems
3. **Improved UX**: Clear feedback prevents user confusion
4. **System Monitoring**: Administrators can observe error patterns
5. **Graceful Degradation**: System continues working during partial failures

This WebSocket error system provides comprehensive real-time error communication for the casino application, ensuring users have immediate feedback on transaction settlements and system status.
