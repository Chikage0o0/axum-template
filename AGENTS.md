# PROJECT_NAME Agent Guide

本指南用于帮助 AI Agent 在 `PROJECT_NAME` 项目中高效、规范地开发。

## 1. 项目概览

- **Backend**: Rust (Axum, Tokio, sqlx)
- **Frontend**: SvelteKit + Svelte 5 + Tailwind CSS (TypeScript)
- **Environment**: Nix + devenv
- **Database**: PostgreSQL (由 devenv 管理)

## 2. 构建与测试

后端（在项目根目录）：

- 构建：`cargo build`
- 运行：`cargo run`
- 测试：`cargo test`
- 格式化：`cargo fmt` / `cargo fmt -- --check`
- Lint：`cargo clippy --all-targets --all-features -- -D warnings`

前端（在 `frontend/`）：

- 安装依赖：`bun install`
- 开发：`bun run dev`
- 类型检查：`bun run check`
- 构建：`bun run build`

环境（在项目根目录）：

- 进入环境：`devenv shell`
- 启动服务：`devenv up`
- 迁移：`db-migrate`

## 3. 必须遵守的工程约定

### API / REST

- **统一前缀**：所有对外 API 必须使用 `/api/v1`。
- **统一错误体**：失败时返回 JSON：`{ code, message, request_id, details? }`。
- **请求链路追踪**：必须支持 `x-request-id`：
  - 客户端可传入 `x-request-id`，服务端透传
  - 客户端未提供时服务端生成并在响应头回传
  - 错误体必须包含 `request_id`

### Rust

- 生产代码禁止 `unwrap()` / `expect()` / `panic!()`；如确有必要必须写中文注释解释原因。
- 跨边界失败（I/O、DB、HTTP）必须保留错误上下文（`anyhow::Context` 或显式 message）。
- SQL 必须参数化（禁止字符串拼接）。

### Svelte 5 / SvelteKit

- 组件内状态优先使用 `$state` / `$derived` / `$effect`，props 使用 `$props()`。
- 禁止从 `$app/stores` 导入（弃用）。统一使用 `$app/state`。

## 4. 文档

- 涉及 API 变更时同步更新 `docs/API.md`。
- 涉及配置项变更时同步更新 `docs/CONFIGURATION.md` 与 `.env.example`。
