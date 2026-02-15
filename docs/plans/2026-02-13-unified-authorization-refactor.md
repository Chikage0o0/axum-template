# Unified Authorization Refactor Implementation Plan

> **For Agents:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将当前基于 `users.role` 的二元鉴权升级为统一策略授权模型（Subject + Permission + Effect + Scope + Constraint），并在不破坏现有 API 契约（含 `x-request-id` / `request_id`）的前提下平滑迁移。

**Architecture:** 采用“一次切换”方式：先完成策略落库、评估器、统一授权入口、业务路由改造与前端能力化，再通过完整回归后统一切换为 policy 判定，不保留旧 role 判断分支。授权决策需要可追踪（`matched_policy_id` + `request_id`）。

**Tech Stack:** Rust (Axum, Tokio, sqlx), PostgreSQL (JSONB), SvelteKit + TypeScript, OpenAPI + orval。

---

## 默认约束与假设

- 策略主体统一采用 `subject_type + subject_key(TEXT)`：`subject_type in {USER, ROLE}`，`subject_key` 在 `USER` 下为用户 UUID 字符串、在 `ROLE` 下为角色名字符串。
- 保留现有 `users.role` 作为迁移期角色来源，不在一期引入独立角色管理后台。
- 一期 scope 先落地 `ALL` / `SELF` / `ID:<uuid>`，`DEPT:SELF` 先保留语义但不在业务查询中强制启用。
- 约束一期先强制支持 `expire_at`，`ip_range` 预留字段与校验位。
- 按用户确认采用“一次切换”，不新增 `legacy/shadow/enforce` 运行期开关。
- 必须保持 `x-request-id` 透传/回写；所有错误体必须包含 `request_id`。
- `/api/v1/users/me` 在迁移完成后稳定返回 `permissions: string[]`，前端仅基于能力集判断可见性。
- 决策稳定性固定为：`priority DESC` -> `effect`（`DENY` 优先）-> 权限特异度（`users:update` > `users:*` > `*`）-> `policy_id ASC`；相同输入必须得到稳定的 `matched_policy_id`。
- 仓储层权限候选集固定为：请求 `users:update` 时按 `{users:update, users:*, *}` 取数（其余权限同形态扩展），再按统一排序返回。
- 非法 `scope_rule` 对外语义固定 fail-closed：返回 `403 + code=2002`（错误体含 `request_id`）；内部日志再区分 `scope_config_error`。

### Task 1: 建立授权领域模型与评估器（纯内存）

**Files:**
- Create: `src/modules/authorization/mod.rs`
- Create: `src/modules/authorization/model.rs`
- Create: `src/modules/authorization/evaluator.rs`
- Test: `src/modules/authorization/evaluator_tests.rs`
- Modify: `src/modules/mod.rs`

**Step 1: 写失败测试（规则优先级、主体合并与命中输出）**

覆盖以下行为：
- `DENY` 覆盖 `ALLOW`（含同优先级冲突）
- `priority` 高策略覆盖低策略
- `perm_code` 支持通配符（如 `users:*`）
- 同优先级冲突时使用固定 tie-break：`DENY` 优先 -> 权限特异度（`users:update` > `users:*` > `*`）-> `policy_id ASC`
- `constraints.expire_at` 过期后自动失效
- 同一请求可合并评估 `USER(uuid)` + `ROLE(role)` 两类主体策略
- 未命中策略默认拒绝
- 命中时返回稳定的 `matched_policy_id`（重复评估结果一致）

**Step 2: 运行测试验证失败**

Run: `cargo test authorization::evaluator -- --nocapture`
Expected: FAIL（缺少模型/评估器实现）

**Step 3: 最小实现通过测试**

实现：`Policy`、`Effect`、`Constraint`、`EvaluationResult` 与 `evaluate()` 决策管道。

**Step 4: 再次执行测试**

Run: `cargo test authorization::evaluator -- --nocapture`
Expected: PASS

### Task 2: 新增策略表与权限元数据迁移

**Files:**
- Create: `migrations/20260213000100_unified_authorization_schema.sql`
- Create: `src/modules/authorization/repository.rs`
- Test: `src/modules/authorization/repository_tests.rs`

**Step 1: 写失败测试（策略读取与排序）**

测试点：
- 可按主体集合（`{USER, <uuid>}` + `{ROLE, <role>}`）读取策略
- 可按权限候选集取数：`users:update -> {users:update, users:*, *}`
- 结果按固定序返回：`priority DESC` -> `effect`（`DENY` 优先）-> 特异度（exact > namespace wildcard > global wildcard）-> `policy_id ASC`
- 过期策略可按 `expire_at` 被过滤

**Step 2: 运行测试验证失败**

Run: `cargo test authorization::repository -- --nocapture`
Expected: FAIL（缺少数据表或查询）

**Step 3: 编写迁移与初始数据**

迁移包含：
- `sys_permission`（权限元数据）
- `sys_policy`（主体策略，含 `effect`、`scope_rule`、`constraints`、`priority`）
- 必要约束（`subject_type`、`effect` 合法值）
- 必要索引（主体维度、权限维度、过期时间查询）
- 初始策略（映射现有 `admin/user` 行为，主体使用 `subject_type=ROLE` + `subject_key=admin|user`）

repository 查询实现约束：
- 先构造权限候选集（如 `users:update -> {users:update, users:*, *}`），再使用参数化查询取数
- 必须按固定序输出候选策略，避免同优先级下 `matched_policy_id` 漂移

**Step 4: 实现 repository 并通过测试**

Run: `cargo test authorization::repository -- --nocapture`
Expected: PASS

### Task 3: 接入鉴权上下文并提供统一授权入口

**Files:**
- Modify: `src/api/auth.rs`
- Create: `src/modules/authorization/service.rs`
- Create: `src/modules/authorization/context.rs`
- Test: `src/http/router/tests/security.rs`

**Step 1: 写失败测试（401/403 与 request-id 契约不变）**

重点断言：
- Token 无效仍为 `401 + code=1001`
- 权限不足仍为 `403 + code=2002`
- 响应头存在 `x-request-id`
- 错误体包含 `request_id`，且与响应头一致
- 客户端传入 `x-request-id` 时服务端原样透传

**Step 2: 新增授权服务接口**

接口示例：
- `authorize(subjects, perm_code, resource_hint, request_id) -> Decision`
- `Decision { allowed, scope_rule, matched_policy_id, effect }`

**Step 3: 在请求上下文挂载授权能力**

`CurrentUser` 保留必要字段，并向授权服务提供 `subjects = [{USER, <uuid>}, {ROLE, <role>}]`。

**Step 4: 增加决策追踪字段**

在授权入口记录结构化日志字段：`request_id`、`perm_code`、`matched_policy_id`、`effect`。

**Step 5: 运行相关测试**

Run: `cargo test security -- --nocapture`
Expected: PASS

### Task 4: 替换硬编码角色判断（users/settings）

**Files:**
- Modify: `src/modules/users/handlers.rs`
- Modify: `src/modules/settings/handlers.rs`
- Test: `src/http/router/tests/users.rs`
- Test: `src/http/router/tests/settings.rs`

**Step 1: 写失败测试（策略驱动）**

新增/调整用例：
- 非管理员但具备 `users:list` 策略时可访问用户列表
- 用户存在显式 `DENY users:*` 时即使角色允许也返回 403
- 权限不足时错误体继续包含 `request_id`

**Step 2: 将 `ensure_admin` 替换为 `authorize` 调用**

建议权限码：
- `settings:view`, `settings:update`
- `users:me:view`, `users:me:update`
- `users:list`, `users:create`, `users:update`, `users:delete`, `users:restore`

**Step 3: 回归测试**

Run: `cargo test users -- --nocapture`
Run: `cargo test settings -- --nocapture`
Expected: PASS

### Task 5: 实现 Scope 翻译器并用于数据过滤

**Files:**
- Create: `src/modules/authorization/scope.rs`
- Modify: `src/modules/users/handlers.rs`
- Test: `src/modules/authorization/scope_tests.rs`
- Test: `src/http/router/tests/users.rs`

**Step 1: 写失败测试（scope 解析）**

覆盖：
- `ALL -> 无过滤`
- `SELF -> users.id = current_user.id`
- `ID:<uuid> -> 指定 id`
- 非法 scope 固定 fail-closed：`403 + code=2002`（错误响应含 `request_id`，且与响应头 `x-request-id` 一致）
- `users:update/delete/restore` 写操作在 `SELF` 下仅允许操作 `current_user.id`
- `users:update/delete/restore` 写操作在 `ID:<uuid>` 下仅允许操作该 `uuid`

**Step 2: 实现 scope 解析与 SQL 参数化翻译**

确保所有查询保持参数化，禁止字符串拼接 SQL。
解析失败或配置非法时内部标记 `scope_config_error`，但对外统一返回 `403 + code=2002`。

**Step 3: 挂接到 users 查询与写操作路径**

- `GET /api/v1/users` 列表查询根据授权结果附加过滤条件；`SELF` 语义固定为 `users.id = current_user.id`。
- `users:update`、`users:delete`、`users:restore` 路由必须同时校验 scope 与目标资源 id，不匹配时返回 `403 + code=2002`。

**Step 4: 执行测试**

Run: `cargo test authorization::scope -- --nocapture`
Run: `cargo test users -- --nocapture`
Expected: PASS

### Task 6: 前端从“角色判断”切到“能力判断”

**Files:**
- Create: `frontend/src/lib/features/auth/model/permission-set.ts`
- Test: `frontend/src/lib/features/auth/model/permission-set.test.ts`
- Modify: `frontend/src/lib/features/auth/state/auth.ts`
- Modify: `frontend/src/lib/features/auth/state/use-auth-bootstrap.svelte.ts`
- Modify: `frontend/src/lib/app/components/app-sidebar.svelte`
- Modify: `frontend/src/routes/(app)/users/+page.svelte`
- Modify: `src/modules/users/handlers.rs`（确保 `/users/me` 响应包含 `permissions`）

**Step 1: 写失败测试（权限判断函数）**

新增前端单测：
- `can("users:list")` 正确判定
- 支持通配符能力（如 `users:*`）

**Step 2: 后端补齐能力数据来源**

在 `/api/v1/users/me` 响应中返回 `permissions: string[]`，并保持 OpenAPI 定义一致。

**Step 3: 从 auth store 中移除角色分支依赖**

UI 菜单与页面访问提示改为读取权限集合，不再直接判断 `admin/user`，并统一使用 `$app/state`。

**Step 4: 运行前端代码生成、检查与测试**

Run: `task frontend:gen:api`
Run: `task frontend:check`
Run: `bun test`
Expected: PASS

### Task 7: 文档、OpenAPI 与切换回归

**Files:**
- Modify: `docs/API.md`
- Modify: `docs/DATABASE.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `src/api/openapi.rs`

**Step 1: 更新接口与错误语义文档**

补充权限码命名规范、scope 规则、constraint 示例，并明确 `x-request-id`/`request_id` 契约与 `/users/me.permissions` 字段。

**Step 2: 同步 OpenAPI 与前端 SDK**

Run: `task frontend:gen:api`
Expected: 生成结果与后端 schema 一致，无类型漂移。

**Step 3: 一次切换前全量回归**

Run: `cargo test`
Run: `task frontend:check`
Run: `task frontend:build`
Run: `task dev:fmt:check`
Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: 全部 PASS

**Step 4: 提交策略**

每完成一个 Task 后进行一次原子提交，且提交操作统一使用 `git-commit` superpower。

---

## 验收标准

- 后端不再出现 `current_user.role == "admin"` 这类硬编码判断。
- 任意授权决策都可追踪到匹配策略（至少记录 `matched_policy_id` 与 `request_id`）。
- 用户管理与设置模块在策略模式下保持现有 401/403/错误码契约不变，且错误体包含 `request_id`、响应头回传 `x-request-id`。
- `/api/v1/users/me` 稳定返回 `permissions`，前端导航与页面访问提示由权限能力驱动，不依赖二元角色分支。
- OpenAPI 与前端生成 SDK 已同步（已执行 `task frontend:gen:api`）。
- 文档与配置示例同步更新，无文档腐烂。
