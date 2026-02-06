ALTER TABLE users
ADD COLUMN password_hash TEXT;

ALTER TABLE users
ADD CONSTRAINT users_password_hash_requires_username CHECK (
    password_hash IS NULL OR (username IS NOT NULL AND char_length(btrim(username)) > 0)
);

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
ON CONFLICT (username) DO UPDATE
SET password_hash = COALESCE(NULLIF(users.password_hash, ''), EXCLUDED.password_hash),
    is_active = TRUE,
    updated_at = NOW();
