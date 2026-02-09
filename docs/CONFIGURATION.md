# 配置说明

本模板采用“两阶段配置”：

1. **启动期（Bootstrap）配置**：只负责“连库 + 绑定端口”，来自环境变量。
2. **运行期（Runtime）配置**：其余全部来自数据库 `system_config`。

## 启动期配置（环境变量）

- `DATABASE_URL`（必填）：数据库连接串。
- `SERVER__HOST`（可选，默认 `0.0.0.0`）：后端监听地址。
- `SERVER__PORT`（可选，默认 `8080`）：后端监听端口。
- `RUST_LOG`（可选）：日志过滤。

可选开关与 seed 参数：

- `SEED_ADMIN_USERNAME`：初始化管理员用户名（默认 `admin`）。
- `SEED_ADMIN_PASSWORD`：首次初始化管理员密码覆盖值（仅在管理员未设置密码且未命中 legacy 密码迁移时使用）。
- `PROJECT_NAME_AUTO_MIGRATE`：是否启动时自动迁移（默认 `true`，设置 `0/false` 禁用）。
- `PROJECT_NAME_EXPOSE_OPENAPI`：是否暴露 OpenAPI/Swagger UI（默认 debug 开、release 关；支持 `1/true` 强制开启，`0/false` 关闭）。

## 前端静态资源嵌入

- 仅在 **release profile**（如 `cargo build --release` / `cargo run --release`）触发前端构建与嵌入。
- release 构建时，`build.rs` 会先清理 `frontend/build`，再执行 `bun run build`，确保是 clean build。
- release 二进制内嵌前端产物，非 `/api` 路径返回前端页面；API 仅在 `/api/*` 下提供。
- debug/test profile 不做前端构建与嵌入。

## 运行期配置（DB: system_config）

表结构：`system_config(key, value jsonb, description, updated_at)`

核心 key：

- `security.jwt_secret`（必需，缺失时由 seed 自动生成）
- `app.check_interval_secs`（默认 `3600`）
- `app.welcome_message`（默认 `Hello from PROJECT_NAME`）
- `integrations.example_api_base`（默认 `https://example.com/api`）
- `integrations.example_api_key`（默认空字符串）

说明：

- `security.jwt_secret` 为 JWT HS256 签名密钥；仅在密钥泄露等应急场景需要轮换。
- `security.admin_password_hash` 已废弃，仅作为迁移来源保留；当前登录密码存储在 `users.password_hash`。
- 运行期配置读取时会做类型检查，类型错误会导致启动失败。
