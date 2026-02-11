CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE system_config (
    key VARCHAR(128) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

COMMENT ON TABLE system_config IS '运行期配置表 - 存储全局配置（JSONB）';

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(64),
    display_name VARCHAR(128) NOT NULL,
    email VARCHAR(320) NOT NULL,
    phone VARCHAR(32),
    avatar_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    password_hash TEXT,
    auth_version INTEGER NOT NULL DEFAULT 0,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_display_name_non_empty CHECK (char_length(btrim(display_name)) > 0),
    CONSTRAINT users_email_non_empty CHECK (char_length(btrim(email)) > 0),
    CONSTRAINT users_password_hash_requires_username CHECK (
        password_hash IS NULL OR (username IS NOT NULL AND char_length(btrim(username)) > 0)
    )
);

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

CREATE UNIQUE INDEX users_username_active_unique
ON users (username)
WHERE deleted_at IS NULL AND username IS NOT NULL;

CREATE UNIQUE INDEX users_email_active_unique
ON users (email)
WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX users_phone_active_unique
ON users (phone)
WHERE deleted_at IS NULL AND phone IS NOT NULL;

CREATE INDEX idx_users_deleted_at ON users (deleted_at);
CREATE INDEX idx_users_active_created_at ON users (created_at DESC)
WHERE deleted_at IS NULL;

CREATE INDEX idx_auth_sessions_user_id ON auth_sessions (user_id);
CREATE INDEX idx_auth_sessions_expires_at ON auth_sessions (expires_at);

INSERT INTO system_config (key, value, description)
VALUES
  ('app.check_interval_secs', '3600'::jsonb, '示例：检查间隔（秒）'),
  ('app.welcome_message', '"Hello from PROJECT_NAME"'::jsonb, '示例：欢迎语')
ON CONFLICT (key) DO NOTHING;

INSERT INTO system_config (key, value, description)
VALUES
  ('integrations.example_api_base', '"https://example.com/api"'::jsonb, '示例：外部 API Base URL'),
  ('integrations.example_api_key', '""'::jsonb, '示例：外部 API Key（留空表示未设置）')
ON CONFLICT (key) DO NOTHING;

WITH legacy_hash AS (
    SELECT trim(btrim(value::text, '"')) AS password_hash
    FROM system_config
    WHERE key = 'security.admin_password_hash'
      AND jsonb_typeof(value) = 'string'
      AND char_length(trim(btrim(value::text, '"'))) > 0
)
INSERT INTO users (
    username,
    display_name,
    email,
    is_active,
    metadata,
    password_hash
)
SELECT
    'admin',
    'Administrator',
    'admin@local.invalid',
    TRUE,
    '{}'::jsonb,
    legacy_hash.password_hash
FROM legacy_hash
ON CONFLICT (username) WHERE deleted_at IS NULL AND username IS NOT NULL DO UPDATE
SET password_hash = COALESCE(NULLIF(users.password_hash, ''), EXCLUDED.password_hash),
    is_active = TRUE,
    updated_at = NOW();
