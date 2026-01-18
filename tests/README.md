# Testing Infrastructure - Phase 4

This directory contains comprehensive tests for the Atomik Wallet betting system.

## Test Structure

```
tests/
├── load/                      # Load testing with k6
│   ├── run-load-test.sh      # Load test runner script
│   └── load-test.js          # k6 load test scenarios
│
services/backend/tests/
├── common/                    # Shared test utilities
│   └── mod.rs                # Test context, fixtures, helpers
├── error_scenarios.rs         # Error handling integration tests
│
services/processor/tests/
├── batch_processing.rs        # Batch processing tests
└── rpc_failover.rs           # RPC failover and resilience tests
```

## Running Tests

### 1. Unit Tests

Run unit tests for each service:

```bash
# Backend unit tests
cd services/backend
cargo test --lib

# Processor unit tests
cd services/processor
cargo test --lib

# Shared crate tests
cd services/shared
cargo test
```

### 2. Integration Tests

**Prerequisites:**
- Backend and Processor services must be running
- Redis must be running on localhost:6379
- Use Redis database 1 for tests (will be automatically flushed)

```bash
# Start services (in separate terminal)
bash scripts/start-services.sh

# Run backend integration tests
cd services/backend
cargo test --test error_scenarios

# Run processor integration tests  
cd services/processor
cargo test --test batch_processing
cargo test --test rpc_failover
```

**Environment Variables:**
```bash
export BACKEND_URL=http://localhost:3001  # Backend URL
export REDIS_URL=redis://127.0.0.1:6379/1 # Redis test database
```

### 3. Load Tests

**Prerequisites:**
- Install k6: `brew install k6` (macOS) or visit https://k6.io/docs/get-started/installation/
- Backend service must be running
- Redis and Processor should be running for full system test

```bash
# Run load test with default settings (10 VUs, 30s)
bash tests/load/run-load-test.sh

# Custom load test parameters
VUS=20 DURATION=60s bash tests/load/run-load-test.sh

# Run k6 directly for more options
k6 run --vus 50 --duration 2m tests/load/load-test.js
```

**Load Test Scenarios:**
- Bet creation (random choices and amounts)
- Bet retrieval by ID
- List user bets
- Health checks
- Validation error scenarios (5% of requests)
- Not found scenarios (3% of requests)

**Performance Thresholds:**
- 95% of requests < 500ms
- Error rate < 10%
- Failed requests < 10%

## Test Coverage

### Backend Integration Tests (`error_scenarios.rs`)

✅ **Validation Errors**
- Invalid bet amount (above maximum)
- Missing required fields
- Proper 400 status and error codes

✅ **Not Found Errors**
- Non-existent bet ID
- Proper 404 status and NOT_FOUND_BET code

✅ **Successful Operations**
- Bet creation with valid data
- Bet retrieval by ID
- List user bets with pagination

✅ **Concurrent Operations**
- 10+ concurrent bet creations
- No race conditions or data corruption

✅ **Health Checks**
- Health endpoint returns 200
- Proper response format

### Processor Tests (`batch_processing.rs`)

✅ **Batch Processing**
- Bets added to pending stream
- Multiple bets processed in order
- Batch status transitions

✅ **Error Handling**
- Retry count increments on failure
- Error codes and messages recorded
- Failed bets marked correctly

✅ **Status Transitions**
- Pending → Batched → Submitted → Completed
- Payout and won fields populated

### RPC Failover Tests (`rpc_failover.rs`)

✅ **Retry Logic**
- Automatic retries on failure
- Success after configured retries
- Request counting

✅ **Endpoint Failover**
- Primary endpoint failure
- Automatic switch to backup endpoint
- Load balancing across endpoints

✅ **Circuit Breaker**
- Opens after threshold failures
- Half-open state recovery
- Closes after successful test

✅ **Backoff Strategies**
- Exponential backoff timing
- Maximum backoff limits

✅ **Concurrent Requests**
- Multiple simultaneous RPC calls
- No race conditions

## Load Test Results

Expected metrics (10 VUs, 30s):

```
✅ Load Test Summary
============================================================

📊 Requests:
  Total: ~300-400
  Failed: <10 (< 2%)

⏱️  Response Time:
  Avg: ~50-150ms
  P95: <500ms
  P99: <800ms

🎲 Bet Creation Avg: ~100ms
🔍 Bet Retrieval Avg: ~30ms

❌ Error Rate: <2%
```

## Continuous Integration

Add to CI pipeline (`.github/workflows/test.yml`):

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run unit tests
        run: cargo test --workspace --lib
      
      - name: Start services
        run: bash scripts/start-services.sh
        
      - name: Run integration tests
        run: |
          cargo test --workspace --test '*'
        env:
          REDIS_URL: redis://localhost:6379/1
          BACKEND_URL: http://localhost:3001
      
      - name: Install k6
        run: |
          sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6
      
      - name: Run load tests
        run: bash tests/load/run-load-test.sh
        env:
          VUS: 5
          DURATION: 10s
```

## Troubleshooting

### Tests Fail with "Connection Refused"
- Ensure backend/processor services are running
- Check services are on expected ports (3001, 9091)
- Verify Redis is accessible

### Redis Tests Fail
- Ensure Redis is running: `redis-cli ping`
- Check tests use database 1: `REDIS_URL=redis://127.0.0.1:6379/1`
- Manually flush test DB: `redis-cli -n 1 FLUSHDB`

### Load Tests Show High Error Rates
- Check backend logs for errors
- Verify Redis can handle load
- Increase connection pool sizes
- Check system resources (CPU, memory)

### Processor Tests Timeout
- Ensure processor is running and processing
- Check worker pool size configuration
- Verify Redis stream is accessible

## Next Steps

- [ ] Add code coverage reporting (tarpaulin)
- [ ] Set up mutation testing
- [ ] Add performance regression tests
- [ ] Create chaos engineering tests
- [ ] Add database migration tests
