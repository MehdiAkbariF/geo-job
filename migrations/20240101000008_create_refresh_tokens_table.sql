CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL, -- SHA-256 hash of the random token
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Unique index on active token hashes
CREATE UNIQUE INDEX IF NOT EXISTS idx_refresh_tokens_hash 
ON refresh_tokens (token_hash);

-- Fast lookup for user sessions
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user 
ON refresh_tokens (user_id);