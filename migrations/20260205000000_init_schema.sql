-- PROJECT_NAME template schema (minimal)

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE system_config (
    key VARCHAR(128) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

COMMENT ON TABLE system_config IS '运行期配置表 - 存储全局配置（JSONB）';
