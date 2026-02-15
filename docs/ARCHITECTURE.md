# 架构概览

本模板的目标不是提供业务功能，而是提供可复用的工程骨架：

- Axum REST API（统一 `/api/v1`）
- `x-request-id` 全链路
- 统一错误体（JSON）
- JWT 鉴权（Bearer Token）
- 策略授权（Policy-based Authorization）
- 两阶段配置：启动期 env + 运行期 DB（`system_config`）
- OpenAPI 作为规范中心（后端导出，前端生成类型与 API 调用函数）

## 分层

```
Frontend (SvelteKit)  ->  Backend API (Axum)  ->  PostgreSQL (sqlx)
```

### 后端

- `src/http/router.rs`：统一路由注册、鉴权路由分组、OpenAPI/静态资源暴露
- `src/http/*`：HTTP 入口层（路由与中间件相关）
- `src/modules/*/handlers.rs`：按业务模块聚合的 handler（sessions/settings/security/users）
- `src/modules/authorization/*`：授权领域（模型、评估器、repository、service、scope、permission enum、字典接口）
- `src/api/request_id.rs`：`x-request-id` 生成/透传/回传，并提供 task-local 访问
- `src/api/openapi.rs`：OpenAPI 文档聚合与 schema 导出
- `src/error.rs`：统一错误枚举与 JSON 序列化
- `src/config/*`：Bootstrap/Runtime/Seed
- `src/services/system_config.rs`：`system_config` 批量 upsert（事务）

### 前端

- `frontend/src/routes/(public)/*`：公开路由（如登录）
- `frontend/src/routes/(app)/*`：登录后应用路由（如仪表盘、设置）
- `frontend/src/lib/features/*`：按业务能力组织的状态与模型（当前含 `auth`）
- `frontend/src/lib/features/auth/model/permission-set.ts`：权限集合判定（精确权限 + 命名空间通配符 + 全局通配符）
- `frontend/src/lib/features/auth/model/permission-node-catalog.ts`：权限节点字典 -> 前端配置选项映射
- `frontend/src/lib/app/*`：应用壳层组件（导航、侧边栏、用户菜单）
- `frontend/src/lib/shared/*`：跨业务复用能力（表单、通用组件、工具）
- `frontend/src/lib/api/generated/*`：由 OpenAPI 生成的客户端与 schema（仅生成，不手改）

## 授权链路

1. `auth_middleware` 完成 Token 校验并构造 `CurrentUser`
2. 业务 handler 调用统一 `authorize(...)`（按 `PermissionNode` 判定）
3. `AuthorizationService` 从 `sys_policy` 读取候选策略并交给评估器稳定排序决策
4. 决策日志记录 `request_id`、`perm_code`、`matched_policy_id`、`effect`
5. 用户域接口按 scope 执行资源过滤/目标校验（`ALL` / `SELF` / `ID:<uuid>`）
6. 前端通过 `/api/v1/users/me.permissions` 做能力判断，不再依赖二元角色分支

## 权限节点单一真源

- 后端 `src/modules/authorization/permission.rs` 仅定义 `PermissionNode` enum（权限码强类型与鉴权引用）。
- `GET /api/v1/authorization/permission-nodes` 在通过登录鉴权后，还要求 `authorization:permission-nodes:view`，并以 `sys_permission` 为数据源返回 `code/name/description/module`，用于前端权限配置 UI/表单渲染。
- `AuthorizationService::list_allowed_permissions` 仅基于 `PermissionNode::ALL` 生成快照，避免数据库中临时/脏权限码污染前端能力集。
