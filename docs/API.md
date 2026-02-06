# API 约定

## 基础约定

- Base：`/api/v1`
- Content-Type：请求/响应以 JSON 为主
- 错误体：`{ code, message, request_id, details? }`
- 追踪：`x-request-id` 必须在响应头回传

## 认证

### 登录

`POST /api/v1/sessions`

请求：

```json
{ "password": "..." }
```

响应：

```json
{ "token": "...", "expires_in": 86400 }
```

## 运行期配置

### 获取配置

`GET /api/v1/settings`（需要 Bearer Token）

### 更新配置

`PATCH /api/v1/settings`（需要 Bearer Token）

说明：更新后会写入 DB `system_config` 并立即热更新到内存。

## 安全

### 修改管理员密码

`PATCH /api/v1/security/admin-password`（需要 Bearer Token）

修改成功后会轮换 `security.jwt_secret`，旧 token 立即失效。
