# 架构概览

本模板的目标不是提供业务功能，而是提供可复用的工程骨架：

- Axum REST API（统一 `/api/v1`）
- `x-request-id` 全链路
- 统一错误体（JSON）
- JWT 鉴权（Bearer Token）
- 两阶段配置：启动期 env + 运行期 DB（`system_config`）
- OpenAPI 作为规范中心（后端导出，前端生成类型与 API 调用函数）

## 分层

```
Frontend (SvelteKit)  ->  Backend API (Axum)  ->  PostgreSQL (sqlx)
```

### 后端

- `src/api/request_id.rs`：`x-request-id` 生成/透传/回传，并提供 task-local 访问
- `src/error.rs`：统一错误枚举与 JSON 序列化
- `src/api/validation.rs`：JSON body + garde 校验的统一 extractor
- `src/config/*`：Bootstrap/Runtime/Seed
- `src/services/system_config.rs`：`system_config` 批量 upsert（事务）
