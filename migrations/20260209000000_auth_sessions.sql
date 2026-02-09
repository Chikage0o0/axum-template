ALTER TABLE users
ADD COLUMN auth_version INTEGER NOT NULL DEFAULT 0;

CREATE TABLE auth_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_secret_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    revoked_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_auth_sessions_user_id ON auth_sessions (user_id);
CREATE INDEX idx_auth_sessions_expires_at ON auth_sessions (expires_at);
