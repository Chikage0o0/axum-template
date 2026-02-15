# 数据库（模板）

本模板包含以下核心表：`system_config`、`users`、`auth_sessions`、`sys_permission`、`sys_policy`。

## SQL 开发约束

- 后端业务代码中的 SQL 必须使用 `sqlx` 宏进行编译时校验（如 `query!`、`query_as!`、`query_scalar!`、`query_file!`）。
- 禁止在业务代码中使用仅运行时校验的 `sqlx::query(...)` / `sqlx::query_as(...)` 形式。

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
- `auth_version` (int, non-null, default `0`)
- `display_name` (varchar, non-null)
- `email` (varchar, unique, non-null)
- `phone` / `avatar_url` (nullable)
- `is_active` (bool)
- `metadata` (jsonb)
- `created_at` / `updated_at` (timestamptz)

用途：

- 存储用户基本信息（本地用户主表）
- `password_hash` 用于本地用户名/密码登录；为空表示仅支持外部身份登录
- `auth_version` 用于用户级凭证版本控制（改密后递增，旧 access token 立即失效）

## 表：auth_sessions

字段（核心）：

- `id` (uuid, PK，会话 ID)
- `user_id` (uuid, FK -> users.id, on delete cascade)
- `refresh_secret_hash` (text, Argon2id PHC)
- `expires_at` (timestamptz)
- `revoked_at` / `revoked_reason` (nullable)
- `created_at` / `updated_at` (timestamptz)

用途：

- 存储 refresh token 对应的服务端会话状态
- 支持 refresh token 轮换（rotation）与会话撤销
- 支持“仅当前用户全部设备下线”，不影响其他用户

## 表：sys_permission

字段（核心）：

- `perm_code` (text, PK)
- `perm_name` (text)
- `description` (text, nullable)
- `created_at` / `updated_at` (timestamptz)

用途：

- 维护权限码元数据（OpenAPI/前端能力化与后端策略评估的共同字典）

## 表：sys_policy

字段（核心）：

- `policy_id` (bigserial, PK)
- `subject_type` (text, `USER|ROLE`)
- `subject_key` (text)
- `perm_code` (text, FK -> `sys_permission.perm_code`)
- `effect` (text, `ALLOW|DENY`)
- `scope_rule` (text, default `ALL`)
- `constraints` (jsonb, default `{}`)
- `expire_at` (timestamptz, nullable)
- `priority` (int, default `0`)
- `created_at` / `updated_at` (timestamptz)

约束与索引：

- `subject_type` / `effect` 均有枚举值约束
- `constraints` 要求为 JSON object，且 `expire_at`、`ip_range`（若存在）必须是字符串
- 关键索引：`(subject_type, subject_key)`、`(perm_code)`、`(expire_at)`

用途：

- 承载统一策略授权（按主体、权限、效果、范围、约束进行判定）
- 支持同一请求合并评估 `USER` 与 `ROLE` 策略
