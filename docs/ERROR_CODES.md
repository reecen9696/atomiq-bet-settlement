# Error Code Reference

This document provides a comprehensive reference for all error codes used in the atomik-wallet system.

## Error Categories

All errors are categorized by domain to enable consistent handling and appropriate HTTP status codes:

| Category       | HTTP Status | Log Level | Description                             |
| -------------- | ----------- | --------- | --------------------------------------- |
| `VALIDATION`   | 400         | warn      | Client provided invalid input           |
| `NETWORK`      | 503         | error     | External service unavailable or timeout |
| `CONTRACT`     | 400         | warn      | Smart contract execution failed         |
| `INTERNAL`     | 500         | error     | Unexpected internal failures            |
| `NOT_FOUND`    | 404         | info      | Requested resource not found            |
| `UNAUTHORIZED` | 401         | warn      | Authentication/authorization failed     |

## Error Code Format

Error codes follow the pattern: `<CATEGORY>_<SPECIFIC>_<DETAIL>`

Examples:

- `VALIDATION_INVALID_BET_ID`
- `NETWORK_RPC_UNAVAILABLE`
- `CONTRACT_EXECUTION_FAILED`

## Validation Errors (400 Bad Request)

### VALIDATION_INVALID_BET_ID

**Description**: Bet ID is malformed or exceeds maximum length (32 characters)

**Context**:

- Bet ID must be alphanumeric, max 32 chars
- Used when deserializing bet creation requests

**Example**:

```json
{
  "error": {
    "code": "VALIDATION_INVALID_BET_ID",
    "message": "Invalid bet ID: abc-def-ghi-jkl-mno-pqr-stu-vwx-yz-123",
    "category": "Validation"
  }
}
```

### VALIDATION_INVALID_AMOUNT

**Description**: Bet amount is outside acceptable range

**Context**:

- Minimum bet: 10,000,000 lamports (0.01 SOL)
- Maximum bet: 1,000,000,000 lamports (1 SOL)
- Amount must be positive

**Example**:

```json
{
  "error": {
    "code": "VALIDATION_INVALID_AMOUNT",
    "message": "Invalid amount: 5000000",
    "category": "Validation"
  }
}
```

### VALIDATION_INVALID_WALLET

**Description**: Wallet address is not a valid Solana public key

**Context**:

- Must be base58-encoded 32-byte public key
- Validated during bet creation

### VALIDATION_INVALID_CHOICE

**Description**: Bet choice is invalid for the game type

**Context**:

- Coinflip: must be "heads" or "tails"

### VALIDATION_INSUFFICIENT_BALANCE

**Description**: User wallet does not have sufficient funds

**Context**:

- Checked before creating allowance or placing bet
- Includes required amount and available balance in context

**Example**:

```json
{
  "error": {
    "code": "VALIDATION_INSUFFICIENT_BALANCE",
    "message": "Insufficient balance",
    "category": "Validation"
  }
}
```

### VALIDATION_ALLOWANCE_EXPIRED

**Description**: User's allowance has expired

**Context**:

- Allowances expire after MAX_ALLOWANCE_DURATION_SECS (86400 seconds / 24 hours)
- User must create a new allowance

## Network Errors (503 Service Unavailable)

### NETWORK_RPC_UNAVAILABLE

**Description**: Solana RPC endpoint is unavailable

**Context**:

- All RPC endpoints in the pool are unhealthy
- Circuit breaker may be open
- Includes endpoint URL in context

### NETWORK_RPC_TIMEOUT

**Description**: Solana RPC request timed out

**Context**:

- Request exceeded configured timeout (typically 30s)
- May indicate network congestion

### NETWORK_REDIS_CONNECTION

**Description**: Redis connection error

**Context**:

- Cannot connect to Redis server
- Redis commands failed
- Includes underlying error in context

### NETWORK_DATABASE_CONNECTION

**Description**: Database connection error

**Context**:

- PostgreSQL connection failed
- Query execution timed out

### NETWORK_BACKEND_UNAVAILABLE

**Description**: Backend API is unavailable

**Context**:

- Processor cannot reach backend API
- Used when processor fetches pending bets

## Smart Contract Errors (400/500)

### CONTRACT_EXECUTION_FAILED

**Description**: Smart contract instruction execution failed

**Context**:

- Transaction was sent but rejected by program
- Includes transaction signature and program error
- Common causes: insufficient rent, invalid PDAs, unauthorized signer

**Example**:

```json
{
  "error": {
    "code": "CONTRACT_EXECUTION_FAILED",
    "message": "Smart contract execution failed",
    "category": "Contract"
  }
}
```

### CONTRACT_INSUFFICIENT_RENT

**Description**: Account has insufficient lamports for rent exemption

**Context**:

- Account would have less than minimum rent-exempt balance after operation
- Prevents accounts from being garbage collected

### CONTRACT_INVALID_PDA

**Description**: PDA derivation does not match expected address

**Context**:

- Program Derived Address mismatch
- Seeds may be incorrect
- Includes expected and actual addresses

### CONTRACT_UNAUTHORIZED_SIGNER

**Description**: Transaction signer is not authorized

**Context**:

- Signer does not match expected authority
- Common with casino or processor operations

### CONTRACT_ACCOUNT_NOT_FOUND

**Description**: Required on-chain account not found

**Context**:

- Vault, casino, or allowance account does not exist
- May need to be initialized first

## Internal Errors (500 Internal Server Error)

### INTERNAL_UNEXPECTED

**Description**: Unexpected internal error

**Context**:

- Generic catch-all for unhandled errors
- Should include stack trace or error details in logs

### INTERNAL_SERIALIZATION

**Description**: Failed to serialize data

**Context**:

- Borsh serialization failed
- JSON serialization failed
- Includes underlying error

### INTERNAL_DESERIALIZATION

**Description**: Failed to deserialize data

**Context**:

- Cannot parse on-chain account data
- Invalid JSON from API response

### INTERNAL_DATABASE_QUERY

**Description**: Database query failed

**Context**:

- SQL query execution error
- Constraint violation
- Includes query details in logs

### INTERNAL_CONFIGURATION

**Description**: Configuration error

**Context**:

- Missing required environment variable
- Invalid configuration value
- Occurs during startup

## Resource Not Found Errors (404 Not Found)

### NOT_FOUND_BET

**Description**: Bet with given ID not found

**Context**:

- Bet ID does not exist in database
- May have been deleted or never created

### NOT_FOUND_BATCH

**Description**: Batch with given ID not found

**Context**:

- Used by processor when updating batch status

### NOT_FOUND_VAULT

**Description**: Vault account not found on-chain

**Context**:

- PDA derivation correct but account not initialized

### NOT_FOUND_ALLOWANCE

**Description**: Allowance account not found

**Context**:

- User has not created allowance yet
- Allowance may have been closed

## Structured Logging

All errors are logged with structured fields for observability:

```rust
tracing::error!(
    error_code = %service_error.code,
    error_category = ?service_error.category,
    error_message = %service_error.message,
    error_context = ?service_error.context,
    "Request failed with error"
);
```

### Log Levels by Category

- `VALIDATION`: warn level (user error, not system issue)
- `NETWORK`: error level (system dependency failure)
- `CONTRACT`: warn level (expected failure mode)
- `INTERNAL`: error level (requires investigation)
- `NOT_FOUND`: info level (normal operation)
- `UNAUTHORIZED`: warn level (potential security issue)

## Metrics

All errors increment the `errors_total` counter with labels:

```
errors_total{category="Validation",code="VALIDATION_INVALID_AMOUNT"} 42
errors_total{category="Network",code="NETWORK_RPC_UNAVAILABLE"} 5
errors_total{category="Internal",code="INTERNAL_UNEXPECTED"} 1
```

## Error Response Format

All API errors return a consistent JSON structure:

```json
{
  "error": {
    "code": "VALIDATION_INVALID_BET_ID",
    "message": "Invalid bet ID: abc123",
    "category": "Validation"
  }
}
```

Context field is omitted from API responses for security but included in logs.

## Usage Examples

### Backend Handler

```rust
// Validation error with context
if bet_amount < MIN_BET_LAMPORTS {
    return Err(AppError::Service(
        ServiceError::invalid_amount(bet_amount, "below minimum")
    ));
}

// Not found error
let bet = repo.find_by_id(bet_id)
    .await?
    .ok_or_else(|| AppError::not_found(format!("Bet {} not found", bet_id)))?;
```

### Processor Worker

```rust
// Network error with endpoint
let client = pool.get_healthy_client()
    .ok_or_else(|| ServiceError::rpc_unavailable(&endpoint))?;

// Contract execution error
match send_transaction(&tx).await {
    Err(e) => {
        return Err(ServiceError::contract_execution_failed(
            signature,
            format!("{:?}", e)
        ));
    }
}
```

## Adding New Error Codes

1. Add error code constant to `shared/src/errors.rs`:

   ```rust
   pub const VALIDATION_MY_NEW_ERROR: ErrorCode = ErrorCode("VALIDATION_MY_NEW_ERROR");
   ```

2. Add constructor method to `ServiceError`:

   ```rust
   pub fn my_new_error(param: impl fmt::Display) -> Self {
       Self::new(
           ErrorCategory::Validation,
           ErrorCode::VALIDATION_MY_NEW_ERROR,
           format!("My error: {}", param),
       )
   }
   ```

3. Document in this file with:
   - Description
   - Context/when it occurs
   - Example JSON response
   - Example usage

4. Add test case to `shared/src/errors.rs` tests section
