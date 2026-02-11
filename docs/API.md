# API 约定

## 基础约定

- Base：`/api/v1`
- Content-Type：请求/响应以 JSON 为主（`GET /api/v1/health` 返回 `text/plain`）
- 错误体：`{ code, message, request_id, details? }`
- 追踪：客户端可传 `x-request-id`，服务端透传；未传时自动生成并在响应头回传
- 非 `/api` 路径由前端页面接管（仅 release 构建）

## 基础设施接口

### 健康检查

- `GET /api/v1/health`
- 返回 `200 OK`，响应体：`OK`

### OpenAPI / Swagger（可选暴露）

- `GET /api/v1/openapi.json`
- `GET /api/v1/swagger-ui`
- 由 `PROJECT_NAME_EXPOSE_OPENAPI` 控制暴露（默认 debug 开、release 关）

### 错误码

- `1000`：参数验证失败
- `1001`：令牌问题（缺少 Bearer Token、Token 无效或已过期）
- `1002`：凭证问题（用户名/密码错误、当前密码错误）
- `2000`：资源不存在
- `2002`：权限不足
- `5000`：内部错误

## 认证

### 登录

`POST /api/v1/sessions`

请求示例：

```json
{ "identifier": "admin", "password": "..." }
```

响应示例：

```json
{ "token": "...", "expires_in": 900 }
```

说明：

- `identifier` 支持 `邮箱`、`用户名`、`手机号`
- `expires_in` 固定为 15 分钟（`900` 秒）
- 响应会通过 `Set-Cookie` 写入 HttpOnly `refresh_token`（有效期 30 天）

### 刷新会话

`POST /api/v1/sessions/refresh`

说明：

- 通过 HttpOnly `refresh_token` 轮换并签发新的 access token
- 响应体同登录：`{ "token": "...", "expires_in": 900 }`

### 退出当前会话

`DELETE /api/v1/sessions/current`（需要 Bearer Token）

响应：`204 No Content`。

## 运行期配置

以下接口均需要 Bearer Token。

### 获取配置

`GET /api/v1/settings`

返回字段：

- `app.check_interval_secs`
- `app.welcome_message`
- `integrations.example_api_base`
- `integrations.example_api_key_is_set`（仅返回是否已设置，不回传明文 key）

### 更新配置

`PATCH /api/v1/settings`

请求支持部分更新：

- `app.check_interval_secs`（最小值 10）
- `app.welcome_message`（非空字符串）
- `integrations.example_api_base`（非空字符串）
- `integrations.example_api_key`（提供时必须非空；不提供表示不修改）

说明：更新后会写入 `system_config` 并立即热更新内存配置。

## 安全

### 修改当前登录用户密码

`PATCH /api/v1/security/password`（需要 Bearer Token）

请求示例：

```json
{ "current_password": "old", "new_password": "new-password-123" }
```

响应：`204 No Content`。

说明：修改成功后会撤销当前用户全部会话（所有设备需重新登录），但不会影响其他用户。

## 用户管理

以下接口均需要 Bearer Token。

权限说明：除 `GET /api/v1/users/me` 外，用户管理接口均要求 `admin` 角色，非管理员返回 `403`（错误码 `2002`）。

### 获取当前登录用户

`GET /api/v1/users/me`

返回当前 Bearer Token 对应用户信息，字段结构与 `GET /api/v1/users` 列表项一致。

### 获取用户列表

`GET /api/v1/users`

查询参数：

- `include_deleted`（可选，默认 `false`）：`false` 时仅返回未删除用户；`true` 时包含已逻辑删除用户

响应示例：

```json
[
  {
    "id": "2f22d798-196c-4c45-98bc-59ca13f457ab",
    "username": "alice",
    "display_name": "Alice",
    "email": "alice@example.com",
    "phone": null,
    "avatar_url": null,
    "is_active": true,
    "metadata": {},
    "created_at": "2026-02-06T09:00:00Z",
    "updated_at": "2026-02-06T09:00:00Z"
  }
]
```

### 创建用户

`POST /api/v1/users`

请求示例：

```json
{
  "username": "alice",
  "display_name": "Alice",
  "email": "alice@example.com"
}
```

说明：

- `username` 为可选字段
- 若提供 `username`，其值不能与其他未删除用户的 `email` 或 `phone` 相同
- `username` 只能包含字母、数字、下划线，且必须至少包含一个字母，不能包含 `@`

### 更新用户基本信息

`PATCH /api/v1/users/{user_id}`

支持按需更新：`username`、`display_name`、`email`、`phone`、`avatar_url`、`is_active`、`metadata`。

其中 `username` 更新时同样受限：不能与其他未删除用户的 `email` 或 `phone` 相同，且格式规则与创建一致。

注意：至少需要提供一个可更新字段，否则返回参数错误。

### 逻辑删除用户

`DELETE /api/v1/users/{user_id}`

响应：`204 No Content`。

说明：该操作为逻辑删除（设置 `deleted_at`），默认用户列表将隐藏该用户。

### 恢复已删除用户

`POST /api/v1/users/{user_id}/restore`

响应：`200 OK`，返回恢复后的用户对象。
