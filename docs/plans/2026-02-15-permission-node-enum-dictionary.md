# Permission Node Enum & Dictionary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将接口权限节点收敛为后端强类型 `enum`，并新增权限节点字典接口，保证前端可稳定拉取并配置权限项。

**Architecture:** 以 `src/modules/authorization/permission.rs` 作为权限节点唯一真源（code/name/description/module），业务鉴权调用从裸字符串迁移到 `enum`。新增受保护接口 `GET /api/v1/authorization/permission-nodes` 返回字典数据，并把该接口纳入 OpenAPI，驱动前端 SDK 生成与配置模型。前端本期只建设“可复用的数据源层”，不在本期新增完整策略编辑 UI（YAGNI）。

**Tech Stack:** Rust (Axum, sqlx, utoipa), PostgreSQL, SvelteKit + TypeScript, orval。

---

## 执行约束

- 全程按 @superpowers:test-driven-development 执行（先写失败测试，再最小实现，再回归）。
- 每完成一个 Task 后调用 `git-commit` 技能提交一次（小步原子提交，避免堆叠改动）。
- 不手改 `frontend/src/lib/api/generated/*`，统一通过 `task frontend:gen:api` 生成。
- API 错误体继续保持 `{ code, message, request_id, details? }` 契约。

### Task 1: 建立权限节点 Enum 与目录模型（后端真源）

**Files:**
- Create: `src/modules/authorization/permission.rs`
- Create: `src/modules/authorization/permission_tests.rs`
- Modify: `src/modules/authorization/mod.rs`

**Step 1: 写失败测试（枚举解析、稳定顺序、无重复）**

```rust
use super::permission::{PermissionNode, permission_catalog};

#[test]
fn permission_node_should_parse_from_code() {
    assert_eq!(
        PermissionNode::try_from_code("users:list"),
        Some(PermissionNode::UsersList)
    );
    assert_eq!(PermissionNode::try_from_code("unknown:perm"), None);
}

#[test]
fn permission_catalog_should_be_unique_and_stable() {
    let catalog = permission_catalog();
    let codes: Vec<&str> = catalog.iter().map(|item| item.code.as_str()).collect();
    assert_eq!(codes.first().copied(), Some("*"));
    assert!(codes.contains(&"users:list"));

    let mut dedup = codes.clone();
    dedup.sort_unstable();
    dedup.dedup();
    assert_eq!(dedup.len(), codes.len(), "权限 code 不允许重复");
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test authorization::permission -- --nocapture`  
Expected: FAIL（提示 `permission` 模块/类型不存在）

**Step 3: 写最小实现**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub enum PermissionNode {
    All,
    UsersAll,
    UsersMeView,
    UsersMeUpdate,
    UsersList,
    UsersCreate,
    UsersUpdate,
    UsersDelete,
    UsersRestore,
    SettingsView,
    SettingsUpdate,
    SecurityPasswordUpdate,
    SessionsCurrentDelete,
}

impl PermissionNode {
    pub const ALL: [PermissionNode; 13] = [
        PermissionNode::All,
        PermissionNode::UsersAll,
        PermissionNode::UsersMeView,
        PermissionNode::UsersMeUpdate,
        PermissionNode::UsersList,
        PermissionNode::UsersCreate,
        PermissionNode::UsersUpdate,
        PermissionNode::UsersDelete,
        PermissionNode::UsersRestore,
        PermissionNode::SettingsView,
        PermissionNode::SettingsUpdate,
        PermissionNode::SecurityPasswordUpdate,
        PermissionNode::SessionsCurrentDelete,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            PermissionNode::All => "*",
            PermissionNode::UsersAll => "users:*",
            PermissionNode::UsersMeView => "users:me:view",
            PermissionNode::UsersMeUpdate => "users:me:update",
            PermissionNode::UsersList => "users:list",
            PermissionNode::UsersCreate => "users:create",
            PermissionNode::UsersUpdate => "users:update",
            PermissionNode::UsersDelete => "users:delete",
            PermissionNode::UsersRestore => "users:restore",
            PermissionNode::SettingsView => "settings:view",
            PermissionNode::SettingsUpdate => "settings:update",
            PermissionNode::SecurityPasswordUpdate => "security:password:update",
            PermissionNode::SessionsCurrentDelete => "sessions:current:delete",
        }
    }

    pub fn try_from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|item| item.as_str() == code)
    }
}
```

**Step 4: 再次运行测试**

Run: `cargo test authorization::permission -- --nocapture`  
Expected: PASS

**Step 5: 提交**

Run: 调用 Skill 工具 `git-commit`  
Suggested message: `feat(auth): add permission node enum catalog`

### Task 2: 鉴权调用改为 Enum，消除裸字符串

**Files:**
- Modify: `src/api/auth.rs`
- Modify: `src/modules/users/handlers.rs`
- Modify: `src/modules/settings/handlers.rs`
- Modify: `src/modules/authorization/service.rs`
- Test: `src/http/router/tests/users.rs`
- Test: `src/http/router/tests/settings.rs`

**Step 1: 写失败测试（未知权限码 fail-closed + 已知权限正常）**

```rust
#[test]
fn permission_node_should_not_accept_unknown_code() {
    assert!(
        crate::modules::authorization::permission::PermissionNode::try_from_code("users:unknown")
            .is_none()
    );
}
```

并在现有路由测试里补一个断言：`/users/me.permissions` 中仍包含 `users:me:view` 与 `users:me:update`。

**Step 2: 运行测试验证失败**

Run: `cargo test users::get_current_user_should_include_permissions -- --nocapture`  
Expected: FAIL（签名/调用未迁移或权限列表来源未收敛）

**Step 3: 写最小实现（签名改造）**

```rust
pub async fn authorize(
    state: &AppState,
    current_user: &CurrentUser,
    permission: PermissionNode,
    resource_hint: Option<&str>,
) -> Result<Decision, AppError> {
    let request_id = current_request_id().unwrap_or_else(|| "req_unknown".to_string());
    let ctx = current_user.authorization_context();
    let decision = state
        .authorization_service
        .authorize(ctx.subjects(), permission.as_str(), resource_hint, &request_id)
        .await?;
    if !decision.allowed {
        return Err(AppError::PermissionDenied("权限不足".to_string()));
    }
    Ok(decision)
}
```

并将业务调用改为：

```rust
authorize(&state, &current_user, PermissionNode::UsersList, None).await?;
```

`list_allowed_permissions` 迭代 `PermissionNode::ALL`（而不是 DB 动态列表），输出 `permission.as_str()`。

**Step 4: 运行测试验证通过**

Run: `cargo test users -- --nocapture`  
Run: `cargo test settings -- --nocapture`  
Expected: PASS

**Step 5: 提交**

Run: 调用 Skill 工具 `git-commit`  
Suggested message: `refactor(auth): switch authorization calls to permission enum`

### Task 3: 新增权限节点字典接口（后端 + OpenAPI）

**Files:**
- Create: `src/modules/authorization/handlers.rs`
- Create: `src/http/router/tests/authorization.rs`
- Modify: `src/modules/authorization/mod.rs`
- Modify: `src/http/router.rs`
- Modify: `src/api/openapi.rs`

**Step 1: 写失败测试（401/200 与字典结构）**

```rust
#[sqlx::test(migrations = "./migrations")]
async fn permission_nodes_dictionary_should_require_auth(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool).await;
    let response = request_json(
        &server,
        Method::GET,
        "/api/v1/authorization/permission-nodes",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
```

再补一个已登录用例，断言：`items` 非空、存在 `users:list`、字段包含 `code/name/description/module`。

**Step 2: 运行测试验证失败**

Run: `cargo test permission_nodes_dictionary -- --nocapture`  
Expected: FAIL（路由或 handler 尚未实现）

**Step 3: 写最小实现（handler + route + OpenAPI）**

```rust
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PermissionNodeItem {
    pub code: PermissionNode,
    pub name: &'static str,
    pub description: &'static str,
    pub module: &'static str,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PermissionNodeDictionaryResponse {
    pub version: &'static str,
    pub items: Vec<PermissionNodeItem>,
}

pub async fn list_permission_nodes_handler() -> Result<axum::Json<PermissionNodeDictionaryResponse>, AppError> {
    Ok(axum::Json(PermissionNodeDictionaryResponse {
        version: "2026-02-15",
        items: permission_catalog()
            .into_iter()
            .map(|item| PermissionNodeItem {
                code: item.code,
                name: item.name,
                description: item.description,
                module: item.module,
            })
            .collect(),
    }))
}
```

路由挂载到受保护路由组：`GET /api/v1/authorization/permission-nodes`。  
OpenAPI 注册新 path 与 schema，并新增 path 暴露测试。

**Step 4: 运行测试验证通过**

Run: `cargo test permission_nodes_dictionary -- --nocapture`  
Run: `cargo test openapi -- --nocapture`  
Expected: PASS

**Step 5: 提交**

Run: 调用 Skill 工具 `git-commit`  
Suggested message: `feat(auth): add permission nodes dictionary endpoint`

### Task 4: 前端接入字典数据源（用于权限配置表单）

**Files:**
- Create: `frontend/src/lib/features/auth/model/permission-node-catalog.ts`
- Create: `frontend/src/lib/features/auth/model/permission-node-catalog.test.ts`
- Modify: `frontend/src/lib/api/generated/client.ts`（通过生成）
- Modify: `frontend/src/lib/api/generated/schemas.ts`（通过生成）

**Step 1: 写失败测试（前端解析与分组）**

```ts
import { describe, expect, it } from "bun:test";
import { toPermissionOptions } from "./permission-node-catalog";

describe("toPermissionOptions", () => {
  it("应把字典项映射为配置选项", () => {
    const options = toPermissionOptions({
      version: "2026-02-15",
      items: [
        {
          code: "users:list",
          name: "List Users",
          description: "查看用户列表",
          module: "users",
        },
      ],
    });
    expect(options[0]?.value).toBe("users:list");
    expect(options[0]?.label).toContain("List Users");
  });
});
```

**Step 2: 运行测试验证失败**

Run (in `frontend/`): `bun test src/lib/features/auth/model/permission-node-catalog.test.ts`  
Expected: FAIL（模块或函数不存在）

**Step 3: 生成 SDK 并写最小实现**

Run: `task frontend:gen:api`

```ts
import { getPermissionNodesHandler } from "$lib/api/generated/client";

export type PermissionOption = {
  value: string;
  label: string;
  module: string;
  description: string;
};

export function toPermissionOptions(input: {
  version: string;
  items: { code: string; name: string; description: string; module: string }[];
}): PermissionOption[] {
  return input.items.map((item) => ({
    value: item.code,
    label: `${item.name} (${item.code})`,
    module: item.module,
    description: item.description,
  }));
}

export async function loadPermissionOptions(): Promise<PermissionOption[]> {
  const dictionary = await getPermissionNodesHandler();
  return toPermissionOptions(dictionary);
}
```

**Step 4: 运行测试与类型检查**

Run (in `frontend/`): `bun test src/lib/features/auth/model/permission-node-catalog.test.ts`  
Run: `task frontend:check`  
Expected: PASS

**Step 5: 提交**

Run: 调用 Skill 工具 `git-commit`  
Suggested message: `feat(frontend): add permission node catalog data source`

### Task 5: 文档与最终回归

**Files:**
- Modify: `docs/API.md`
- Modify: `docs/ARCHITECTURE.md`

**Step 1: 写失败校验（文档约束清单）**

人工检查清单（先记录为 TODO，未完成即视为失败）：
- API 文档新增 `GET /api/v1/authorization/permission-nodes`
- 明确“权限节点以后端 enum 为唯一真源”
- 明确前端应通过字典接口配置权限选项，不硬编码

**Step 2: 更新文档**

在 `docs/API.md` 增加：

```md
### 权限节点字典

- `GET /api/v1/authorization/permission-nodes`
- 返回：`{ version, items[] }`
- `items[]` 字段：`code`、`name`、`description`、`module`
- 约束：`code` 来自后端 `PermissionNode` enum（OpenAPI enum 可解析）
```

并在 `docs/ARCHITECTURE.md` 增加“权限节点单一真源”说明。

**Step 3: 全量验证**

Run: `cargo test`  
Run: `task frontend:check`  
Run: `task frontend:build`  
Run: `task dev:fmt:check`  
Run: `cargo clippy --all-targets --all-features -- -D warnings`  
Expected: 全部 PASS

**Step 4: 最终提交**

Run: 调用 Skill 工具 `git-commit`  
Suggested message: `docs: document enum-driven permission dictionary contract`

---

## 验收标准

- 后端存在 `PermissionNode` 强类型枚举，业务鉴权调用不再使用权限裸字符串。
- 新增 `GET /api/v1/authorization/permission-nodes`，且接口已在 OpenAPI 暴露。
- 字典返回结构稳定（`version + items`），`code` 可由规范 enum 直接解析。
- 前端可通过生成 SDK + `permission-node-catalog` 数据层加载并转换权限配置选项。
- `docs/API.md` 与 `docs/ARCHITECTURE.md` 已同步，无“实现变更未记文档”的腐烂。
