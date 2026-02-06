CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(64) UNIQUE,
    display_name VARCHAR(128) NOT NULL,
    email VARCHAR(320) NOT NULL UNIQUE,
    phone VARCHAR(32),
    avatar_url TEXT,
    locale VARCHAR(32),
    timezone VARCHAR(64),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_display_name_non_empty CHECK (char_length(btrim(display_name)) > 0),
    CONSTRAINT users_email_non_empty CHECK (char_length(btrim(email)) > 0)
);

CREATE TABLE user_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_kind VARCHAR(32) NOT NULL,
    provider_name VARCHAR(64) NOT NULL,
    provider_user_id VARCHAR(256) NOT NULL,
    provider_username VARCHAR(256),
    provider_email VARCHAR(320),
    oidc_issuer TEXT,
    oidc_subject VARCHAR(256),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    CONSTRAINT user_identities_provider_kind_non_empty CHECK (char_length(btrim(provider_kind)) > 0),
    CONSTRAINT user_identities_provider_name_non_empty CHECK (char_length(btrim(provider_name)) > 0),
    CONSTRAINT user_identities_provider_user_id_non_empty CHECK (char_length(btrim(provider_user_id)) > 0),
    CONSTRAINT user_identities_oidc_subject_requires_issuer CHECK (
        oidc_subject IS NULL OR (oidc_issuer IS NOT NULL AND char_length(btrim(oidc_issuer)) > 0)
    ),
    CONSTRAINT user_identities_user_provider_unique UNIQUE (user_id, provider_kind, provider_name),
    CONSTRAINT user_identities_provider_user_unique UNIQUE (provider_kind, provider_name, provider_user_id)
);

CREATE INDEX idx_users_is_active ON users (is_active);
CREATE INDEX idx_users_created_at ON users (created_at DESC);
CREATE INDEX idx_user_identities_user_id ON user_identities (user_id);
CREATE INDEX idx_user_identities_provider ON user_identities (provider_kind, provider_name);
