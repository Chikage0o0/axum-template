ALTER TABLE users
ADD COLUMN deleted_at TIMESTAMPTZ;

ALTER TABLE users DROP CONSTRAINT IF EXISTS users_username_key;
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_email_key;

DROP INDEX IF EXISTS idx_users_created_at;
DROP INDEX IF EXISTS idx_users_is_active;

CREATE UNIQUE INDEX users_username_active_unique
ON users (username)
WHERE deleted_at IS NULL AND username IS NOT NULL;

CREATE UNIQUE INDEX users_email_active_unique
ON users (email)
WHERE deleted_at IS NULL;

CREATE INDEX idx_users_deleted_at ON users (deleted_at);
CREATE INDEX idx_users_active_created_at ON users (created_at DESC)
WHERE deleted_at IS NULL;
