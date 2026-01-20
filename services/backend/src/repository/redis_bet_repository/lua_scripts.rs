//! Redis Lua scripts for atomic operations
//!
//! Contains Lua script constants used for complex Redis transactions.

/// Lua script to atomically claim pending bets for batch processing
///
/// Keys: [claimable_index, processing_index]
/// Args: [limit, batch_id, processor_id, now_ms]
///
/// Returns: Array of claimed bet IDs
pub const CLAIM_PENDING_SCRIPT: &str = r#"
local claimable = KEYS[1]
local processing = KEYS[2]
local limit = tonumber(ARGV[1])
local batch_id = ARGV[2]
local processor_id = ARGV[3]
local now_ms = tonumber(ARGV[4])

-- Claim only bets that are due (score <= now_ms). Score is treated as "available_at_ms".
local entries = redis.call('ZRANGEBYSCORE', claimable, '-inf', now_ms, 'WITHSCORES', 'LIMIT', 0, limit)
local claimed = {}

for i = 1, #entries, 2 do
  local bet_id = entries[i]
  local score = entries[i + 1]
  redis.call('ZREM', claimable, bet_id)
  redis.call('ZADD', processing, score, bet_id)
  redis.call('HSET', 'bet:' .. bet_id,
    'status', 'batched',
    'external_batch_id', batch_id,
    'processor_id', processor_id
  )
  table.insert(claimed, bet_id)
end

return claimed
"#;

/// Lua script for handling failed retryable bet status updates
///
/// Keys: [bet_key, claimable_index, processing_index]
/// Args: [bet_id, now_ms, max_retries, backoff_ms]
///
/// Returns: [new_status, new_retry_count]
///
/// Increments retry count, applies backoff, or escalates to manual review
pub const FAIL_RETRYABLE_SCRIPT: &str = r#"
local bet_key = KEYS[1]
local claimable = KEYS[2]
local processing = KEYS[3]
local bet_id = ARGV[1]
local now_ms = tonumber(ARGV[2])
local max_retries = tonumber(ARGV[3])
local backoff_ms = tonumber(ARGV[4])

local current_retry = tonumber(redis.call('HGET', bet_key, 'retry_count') or '0')
local new_retry = current_retry + 1

redis.call('HSET', bet_key,
    'retry_count', tostring(new_retry),
    'solana_tx_id', ''
)

-- If exceeded retry budget, stop retrying.
if new_retry > max_retries then
    redis.call('HSET', bet_key,
        'status', 'failed_manual_review'
    )
    redis.call('ZREM', claimable, bet_id)
    redis.call('ZREM', processing, bet_id)
    return { 'failed_manual_review', tostring(new_retry) }
end

local next_attempt_at = now_ms + backoff_ms

redis.call('HSET', bet_key,
    'status', 'failed_retryable',
    'next_attempt_at_ms', tostring(next_attempt_at)
)

redis.call('ZADD', claimable, next_attempt_at, bet_id)
redis.call('ZREM', processing, bet_id)

return { 'failed_retryable', tostring(new_retry) }
"#;

/// Lua script for compare-and-swap status update with versioning
///
/// Keys: [bet_key]
/// Args: [expected_version, new_status]
///
/// Returns: 1 if updated, 0 if version mismatch
pub const CAS_UPDATE_SCRIPT: &str = r#"
local bet_key = KEYS[1]
local expected = tonumber(ARGV[1])
local new_status = ARGV[2]

local current = tonumber(redis.call('HGET', bet_key, 'version') or '0')
if current ~= expected then
  return 0
end

redis.call('HSET', bet_key, 'status', new_status)
redis.call('HINCRBY', bet_key, 'version', 1)
return 1
"#;
