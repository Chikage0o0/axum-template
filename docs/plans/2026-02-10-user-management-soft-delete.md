# User Management (Admin CRUD + Soft Delete) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在现有用户管理基础上补齐“管理员可用”的完整 CRUD，新增逻辑删除与后端恢复能力，且逻辑删除后允许复用 `username`/`email`。

**Architecture:** 后端以 Axum + sqlx 为核心，新增 `deleted_at` 软删除字段并通过部分唯一索引保证“仅未删除用户唯一”；API 增加 `DELETE /users/{id}` 与 `POST /users/{id}/restore`，列表支持 `include_deleted`。前端新增用户管理页（列表/新增/编辑/逻辑删除），恢复能力仅保留在后端 API（不在 UI 暴露）。

**Tech Stack:** Rust (Axum, sqlx, utoipa), PostgreSQL, SvelteKit + Svelte 5, shadcn-svelte, Tailwind CSS。

---

### Task 1: 先写失败的后端行为测试（TDD 起点）

**Files:**
- Modify: `src/http/router.rs`
- Test: `src/http/router.rs`（现有 `#[cfg(test)]` 模块内新增用例）

**Step 1: 写失败用例（删除/恢复/权限/复用）**

```rust
#[tokio::test]
async fn delete_user_should_soft_delete_and_hide_from_default_list() {
    // 1) admin 登录
    // 2) 创建普通用户
    // 3) DELETE /api/v1/users/{id}
    // 4) GET /api/v1/users 断言已删除用户默认不可见
}

#[tokio::test]
async fn deleted_user_email_should_be_reusable() {
    // 1) 删除用户 A
    // 2) 使用相同 email 创建用户 B
    // 3) 断言创建成功
}

#[tokio::test]
async fn non_admin_should_forbidden_on_user_management_routes() {
    // 普通用户访问 GET/POST/PATCH/DELETE /api/v1/users... 返回 403
}

#[tokio::test]
async fn restore_user_should_reactivate_soft_deleted_user() {
    // 1) 删除用户
    // 2) POST /api/v1/users/{id}/restore
    // 3) GET /api/v1/users?include_deleted=true 可见且 deleted_at 为 null（或等价恢复状态）
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test delete_user_should_soft_delete_and_hide_from_default_list -- --nocapture`
Expected: FAIL（提示缺少删除/恢复路由或断言不满足）。

**Step 3: 提交测试基线**

Run: 使用 `superpowers:git-commit`，建议消息：`test(users): add failing tests for soft delete, restore and admin guard`

---

### Task 2: 数据库软删除模型与唯一约束改造

**Files:**
- Create: `migrations/20260210000000_users_soft_delete.sql`
- Modify: `.sqlx/*.json`（通过 prepare 自动更新）

**Step 1: 写迁移 SQL（最小必要）**

```sql
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMPTZ;

ALTER TABLE users DROP CONSTRAINT IF EXISTS users_username_key;
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_email_key;

CREATE UNIQUE INDEX users_username_active_unique
ON users (username)
WHERE deleted_at IS NULL AND username IS NOT NULL;

CREATE UNIQUE INDEX users_email_active_unique
ON users (email)
WHERE deleted_at IS NULL;

CREATE INDEX idx_users_deleted_at ON users (deleted_at);
CREATE INDEX idx_users_active_created_at ON users (created_at DESC)
WHERE deleted_at IS NULL;
```

**Step 2: 执行迁移并验证结构**

Run: `task db:migrate`
Expected: PASS（新迁移成功执行）。

**Step 3: 更新 sqlx 离线缓存**

Run: `cargo sqlx prepare --workspace -- --all-targets --all-features`
Expected: PASS（`.sqlx` 快照更新）。

**Step 4: 提交数据库变更**

Run: 使用 `superpowers:git-commit`，建议消息：`feat(db): add users soft delete column and partial unique indexes`

---

### Task 3: 后端 API（删除/恢复/列表过滤）

**Files:**
- Modify: `src/modules/users/handlers.rs`
- Modify: `src/http/router.rs`
- Modify: `src/api/openapi.rs`
- Test: `src/http/router.rs`

**Step 1: 先补最小接口定义与路由（让编译失败点明确）**

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListUsersQuery {
    #[serde(default)]
    pub include_deleted: bool,
}

pub async fn delete_user_handler(Path(user_id): Path<Uuid>, State(state): State<AppState>)
    -> Result<StatusCode, AppError> {
    soft_delete_user(&state.db, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_user_handler(Path(user_id): Path<Uuid>, State(state): State<AppState>)
    -> Result<Json<UserResponse>, AppError> {
    let user = restore_user(&state.db, user_id).await?;
    Ok(Json(user))
}
```

**Step 2: 实现 SQL 逻辑（软删除/恢复/过滤）**

```rust
// list_users: include_deleted=false 时加 WHERE deleted_at IS NULL
// soft_delete_user: deleted_at=NOW(), is_active=FALSE, updated_at=NOW()
// restore_user: deleted_at=NULL, is_active=TRUE, updated_at=NOW()
```

**Step 3: 更新 OpenAPI 路径注册**

```rust
paths(
    users::get_users_handler,
    users::create_user_handler,
    users::patch_user_handler,
    users::delete_user_handler,
    users::restore_user_handler,
)
```

**Step 4: 跑定向测试**

Run: `cargo test delete_user_should_soft_delete_and_hide_from_default_list -- --nocapture`
Expected: PASS。

**Step 5: 提交 API 变更**

Run: 使用 `superpowers:git-commit`，建议消息：`feat(users): add soft delete, restore endpoint and include_deleted query`

---

### Task 4: 管理员权限收敛（仅 admin 可管理用户）

**Files:**
- Modify: `src/api/auth.rs`
- Modify: `src/modules/users/handlers.rs`
- Test: `src/http/router.rs`

**Step 1: 先写失败断言（普通用户 403）**

```rust
assert_eq!(response.status(), StatusCode::FORBIDDEN);
let body = response_json(response).await;
assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));
```

**Step 2: 扩展 CurrentUser 并实现权限守卫**

```rust
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub role: String,
}

fn ensure_admin(current_user: &CurrentUser) -> Result<(), AppError> {
    if current_user.role != "admin" {
        return Err(AppError::PermissionDenied("仅管理员可执行该操作".to_string()));
    }
    Ok(())
}
```

**Step 3: 在用户管理接口统一调用守卫**

```rust
// GET /users, POST /users, PATCH /users/{id}, DELETE /users/{id}, POST /users/{id}/restore
// 均注入 Extension<CurrentUser> 并 ensure_admin(&current_user)?;
// /users/me 保持不受影响
```

**Step 4: 跑权限测试**

Run: `cargo test non_admin_should_forbidden_on_user_management_routes -- --nocapture`
Expected: PASS。

**Step 5: 提交权限变更**

Run: 使用 `superpowers:git-commit`，建议消息：`feat(auth): enforce admin-only access for user management`

---

### Task 5: 前端用户管理页（管理员可见 + Sheet 新增编辑 + 逻辑删除）

**Files:**
- Create: `frontend/src/routes/(app)/users/+page.svelte`
- Create: `frontend/src/routes/(app)/users/+page.ts`
- Create: `frontend/src/lib/features/auth/model/token-role.ts`
- Modify: `frontend/src/lib/features/auth/model/auth-user.ts`
- Modify: `frontend/src/lib/features/auth/state/auth.ts`
- Modify: `frontend/src/lib/app/components/app-sidebar.svelte`
- Modify: `frontend/src/routes/(app)/+layout.svelte`（breadcrumb 增加 Users 映射）
- Modify: `frontend/src/lib/api/generated/client.ts`（由生成覆盖）
- Modify: `frontend/src/lib/api/generated/schemas.ts`（由生成覆盖）

**Step 1: 先更新 OpenAPI 生成客户端（前端先红再绿）**

Run: `task frontend:gen:api`
Expected: PASS，生成 `deleteUserHandler`/`restoreUserHandler`/`include_deleted` 参数。

**Step 2: 增加 JWT role 解析与菜单权限控制（仅 admin 显示 Users）**

```ts
// token-role.ts
export type AuthRole = "admin" | "user";

export function readRoleFromToken(token: string | null): AuthRole {
  // 仅用于 UI 显隐，不作为安全边界；真正鉴权以后端 403 为准
  // 解析 JWT payload 的 role，异常时回退 "user"
}

// auth.ts
login(token: string) {
  store.set({
    isAuthenticated: true,
    token,
    user: null,
    role: readRoleFromToken(token),
    flash: null,
  });
}

// app-sidebar.svelte
// 仅当 role === "admin" 时渲染 /users 菜单
```

**Step 3: 写用户页最小骨架并打通列表（默认不含已删除）**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { ApiError } from "$lib/api/mutator";
  import { getUsersHandler } from "$lib/api/generated/client";

  let users = $state([]);
  let listLoading = $state(false);
  let listError = $state<string | null>(null);
  let permissionDenied = $state(false);

  // onMount -> getUsersHandler({ include_deleted: false })
  // 403 时显示页内空态，不跳转
</script>
```

**Step 4: 新增/编辑共用 Sheet，并在编辑中暴露 `is_active` 开关**

```ts
type SheetMode = "create" | "edit";
type UserFormDraft = {
  username: string;
  display_name: string;
  email: string;
  phone: string;
  avatar_url: string;
  is_active: boolean;
};

// 新建默认启用
const emptyDraft = (): UserFormDraft => ({
  username: "",
  display_name: "",
  email: "",
  phone: "",
  avatar_url: "",
  is_active: true,
});

await createUserHandler(createPayload);
await patchUserHandler(userId, patchPayload); // 可更新 is_active
```

**Step 5: 明确“禁用”和“删除”两种状态，补充自保护约束**

```ts
// 禁用：patch is_active = false，用户仍在列表
// 删除：deleteUserHandler(userId)，用户从默认列表消失

const isSelfRow = (userId: string) => $auth.user?.sub === userId;

// 约束 1：当前登录账号禁止自删除（删除按钮禁用 + 提示）
// 约束 2：当前登录账号禁止自禁用（is_active 开关禁用 + 提交兜底）
```

**Step 6: 接入导航、面包屑映射与页面标题**

```ts
// +page.ts
export const load: PageLoad = () => ({ pageTitle: "用户管理" });

// (app)/+layout.svelte
// 使用路径映射：/ -> 仪表盘, /settings -> 设置, /users -> 用户管理
```

**Step 7: 验证前端质量**

Run: `task frontend:check && task frontend:build`
Expected: PASS。

**Step 8: 提交前端变更**

Run: 使用 `superpowers:git-commit`，建议消息：`feat(frontend): add user management page with create edit soft-delete flows`

---

### Task 6: API 文档与验收回归

**Files:**
- Modify: `docs/API.md`
- Modify: `docs/openapi.json`（若项目流程要求落盘）

**Step 1: 文档补充接口与语义**

```md
- DELETE /api/v1/users/{user_id}：逻辑删除（204）
- POST /api/v1/users/{user_id}/restore：恢复用户（200）
- GET /api/v1/users?include_deleted=true：包含已删除用户
- 权限：除 /users/me 外均需 admin
```

**Step 2: 全量回归**

Run: `task dev:fmt && cargo test && cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS。

**Step 3: 最终提交**

Run: 使用 `superpowers:git-commit`，建议消息：`docs(api): document admin-only user CRUD with soft delete and restore`

---

## 实施细节约束（执行时必须遵守）

- 统一错误体保持 `{ code, message, request_id, details? }`。
- 新增接口全部使用 `/api/v1` 前缀。
- 生产代码禁止 `unwrap()/expect()/panic!()`。
- SQL 必须参数化，且跨边界错误需保留上下文。
- 前端优先使用 shadcn-svelte 组件，禁止修改 `frontend/src/lib/shadcn`。
- 前端 `/users` 默认请求 `include_deleted=false`；不在 UI 暴露恢复能力。
- 仅 admin 显示 Users 菜单；非 admin 直达 `/users` 显示页内 403 空态，不自动跳转。
- 编辑表单暴露 `is_active`；禁用与删除是两种状态。
- 当前登录账号禁止自禁用与自删除（前端交互禁用 + 提交层兜底）。

## 需求决策记录（来自本次澄清）

- 删除语义：新增 `DELETE /api/v1/users/{user_id}`。
- 范围：前后端全栈。
- 恢复能力：后端提供，前端不暴露。
- 列表规则：默认隐藏已删除，可通过参数包含。
- 权限模型：仅管理员可执行用户管理（`/users/me` 除外）。
- 唯一键策略：逻辑删除后允许复用 `username`/`email`。
- 前端权限显示：仅 admin 显示 Users 菜单，admin 判定来自 JWT 的 `role` claim。
- 路由行为：非 admin 手动访问 `/users` 时显示页内 403 空态，不跳转。
- 表单语义：编辑时暴露 `is_active`，且新建用户默认 `is_active=true`。
- 状态区分：禁用（`is_active=false`）与删除（`deleted_at!=null`）是两种状态。
- 安全兜底：当前登录账号禁止自禁用与自删除。
- 范围收敛：首版不做搜索/筛选，仅实现列表/新增/编辑/禁用/删除。
