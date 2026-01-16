-- Create custom types
CREATE TYPE bet_status AS ENUM (
    'pending',
    'batched',
    'submitted_to_solana',
    'confirmed_on_solana',
    'completed',
    'failed_retryable',
    'failed_manual_review'
);

CREATE TYPE batch_status AS ENUM (
    'created',
    'submitted',
    'confirmed',
    'failed'
);

-- Bets table
CREATE TABLE bets (
    bet_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_wallet VARCHAR(44) NOT NULL,
    vault_address VARCHAR(44) NOT NULL,
    casino_id VARCHAR(100),
    game_type VARCHAR(50) NOT NULL,
    stake_amount BIGINT NOT NULL,
    stake_token VARCHAR(44) NOT NULL,
    choice VARCHAR(50) NOT NULL,
    status bet_status NOT NULL DEFAULT 'pending',
    external_batch_id UUID,
    solana_tx_id VARCHAR(88),
    retry_count INTEGER NOT NULL DEFAULT 0,
    processor_id VARCHAR(100),
    last_error_code VARCHAR(50),
    last_error_message TEXT,
    payout_amount BIGINT,
    won BOOLEAN,
    version INTEGER NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for bets
CREATE INDEX idx_bets_status_created ON bets(status, created_at) WHERE status IN ('pending', 'batched', 'submitted_to_solana');
CREATE INDEX idx_bets_user_created ON bets(user_wallet, created_at DESC);
CREATE INDEX idx_bets_batch_id ON bets(external_batch_id) WHERE external_batch_id IS NOT NULL;
CREATE INDEX idx_bets_solana_tx ON bets(solana_tx_id) WHERE solana_tx_id IS NOT NULL;
CREATE INDEX idx_bets_stuck ON bets(status, updated_at) WHERE status = 'submitted_to_solana';

-- Batches table
CREATE TABLE batches (
    batch_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processor_id VARCHAR(100) NOT NULL,
    status batch_status NOT NULL DEFAULT 'created',
    bet_count INTEGER NOT NULL,
    solana_tx_id VARCHAR(88),
    confirm_slot BIGINT,
    confirm_status VARCHAR(50),
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error_code VARCHAR(50),
    last_error_message TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for batches
CREATE INDEX idx_batches_status_created ON batches(status, created_at);
CREATE INDEX idx_batches_processor ON batches(processor_id, created_at DESC);

-- Audit log table (immutable, append-only)
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    event_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type VARCHAR(50) NOT NULL,
    aggregate_id VARCHAR(100) NOT NULL,
    user_id VARCHAR(100),
    before_state JSONB,
    after_state JSONB,
    metadata JSONB,
    actor VARCHAR(100) NOT NULL
);

-- Prevent updates/deletes on audit log
CREATE RULE no_update AS ON UPDATE TO audit_log DO INSTEAD NOTHING;
CREATE RULE no_delete AS ON DELETE TO audit_log DO INSTEAD NOTHING;

-- Indexes for audit log
CREATE INDEX idx_audit_log_event_time ON audit_log(event_time DESC);
CREATE INDEX idx_audit_log_aggregate ON audit_log(aggregate_id, event_time DESC);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type, event_time DESC);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
CREATE TRIGGER update_bets_updated_at
    BEFORE UPDATE ON bets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_batches_updated_at
    BEFORE UPDATE ON batches
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
