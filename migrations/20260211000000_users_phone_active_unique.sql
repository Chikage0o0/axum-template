CREATE UNIQUE INDEX users_phone_active_unique
ON users (phone)
WHERE deleted_at IS NULL AND phone IS NOT NULL;
