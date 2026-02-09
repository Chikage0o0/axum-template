# PROJECT_NAME Template

> 这是从现有项目抽取的“工程规范模板”，核心业务已移除，仅保留可复用的工程骨架与约定。

## 目标

- **后端**：Rust + Axum，内置 `x-request-id`、统一错误体、JWT 鉴权、参数校验（garde）、OpenAPI（规范中心）、运行期配置（DB: `system_config`）热更新。
- **前端**：SvelteKit + Svelte 5 + Tailwind，包含用户名/密码登录页与设置页（读写 `/api/v1/settings`）。
- **环境**：devenv 一键拉起 PostgreSQL + Rust 工具链 + Bun。

## 快速开始

1) 准备环境变量（仅启动期需要）

```bash
cp .env.example .env
```

2) 进入 devenv 环境并启动服务（Postgres）

```bash
devenv shell
devenv up
```

3) 安装前端依赖（仅首次 / 依赖变更后）

```bash
cd frontend
bun install
```

4) 单体运行（release 构建时内嵌前端）

```bash
cargo run --release
```

开发模式不做前端构建与嵌入；本地联调请使用 `cargo run` + `cd frontend && bun run dev`。

## 验证规范是否生效

- `GET /api/v1/health`：健康检查
- `x-request-id`：无论成功/失败都回传响应头 `x-request-id`，错误体也包含 `request_id`
- 统一错误体：失败时返回 JSON：`{ code, message, request_id, details? }`
- 配置热更新：`PATCH /api/v1/settings` 写入 `system_config` 后立即在内存生效
- 安全：修改管理员密码会轮换 JWT secret，使旧 token 立即失效
- 路由接管（release）：访问任意非 `/api` 路径（如 `/login`、`/settings`）都返回前端页面

## PROJECT_NAME 替换指引

模板里所有“可见名称”均使用 `PROJECT_NAME` 占位。

机器标识（Rust crate/package、DB 默认名等）使用可编译的默认值（例如 `project-name` / `project_name`）。
如果你要彻底重命名，请按你自己的命名规范全局替换：

- `PROJECT_NAME` -> 你的项目展示名
- `project-name` / `project_name` -> 你的项目标识
- 环境变量前缀：`PROJECT_NAME_` -> 你的前缀
