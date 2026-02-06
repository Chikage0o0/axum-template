# 数据库（模板）

本模板包含以下核心表：`system_config`、`users`、`user_identities`。

## 表：system_config

字段：

- `key` (varchar, PK)
- `value` (jsonb)
- `description` (text, nullable)
- `updated_at` (timestamptz)

用途：

- 存储运行期配置，便于后续做 Web Settings 管理界面
- 支持热更新（写 DB 后刷新内存）

## 表：users

字段（核心）：

- `id` (uuid, PK)
- `username` (varchar, unique, nullable)
- `password_hash` (text, nullable, Argon2id PHC)
- `display_name` (varchar, non-null)
- `email` (varchar, unique, non-null)
- `phone` / `avatar_url` (nullable)
- `is_active` (bool)
- `metadata` (jsonb)
- `created_at` / `updated_at` (timestamptz)

用途：

- 存储用户基本信息（本地用户主表）
- `password_hash` 用于本地用户名/密码登录；为空表示仅支持外部身份登录
- 与 `user_identities` 形成 1:N 关系，支持一个用户绑定多个外部身份

## 表：user_identities

字段（核心）：

- `id` (uuid, PK)
- `user_id` (uuid, FK -> users.id, on delete cascade)
- `provider_kind` (varchar, non-null，如 `oidc` / `oauth2`)
- `provider_name` (varchar, non-null，如 `google` / `github`)
- `provider_user_id` (varchar, non-null)
- `provider_username` / `provider_email` (nullable)
- `oidc_issuer` / `oidc_subject` (nullable，OIDC 预留)
- `metadata` (jsonb)
- `last_login_at` / `created_at` / `updated_at` (timestamptz)

关键约束：

- `UNIQUE (user_id, provider_kind, provider_name)`：同一用户同一 provider 只能绑定一次
- `UNIQUE (provider_kind, provider_name, provider_user_id)`：同一外部账号只能归属一个用户
