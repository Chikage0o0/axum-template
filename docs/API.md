# API 约定

## 基础约定

- Base：`/api/v1`
- Content-Type：请求/响应以 JSON 为主
- 错误体：`{ code, message, request_id, details? }`
- 追踪：`x-request-id` 必须在响应头回传

### 认证相关错误码

- `1001`：令牌问题（缺少 Bearer Token、Token 无效或已过期）
- `1002`：凭证问题（用户名/密码错误、当前密码错误）

## 认证

### 登录

`POST /api/v1/sessions`

请求：

```json
{ "username": "admin", "password": "..." }
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

### 修改当前用户密码

`PATCH /api/v1/security/password`（需要 Bearer Token）

修改成功后会轮换 `security.jwt_secret`，旧 token 立即失效。

## 用户管理

以下接口均需要 Bearer Token，统一前缀 `/api/v1`。

### 获取当前登录用户

`GET /api/v1/users/me`

用于返回当前 Bearer Token 对应的用户信息，字段结构与 `GET /api/v1/users` 列表项一致。

### 获取用户列表

`GET /api/v1/users`

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
    "identities": [],
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
  "email": "alice@example.com",
  "identities": [
    {
      "provider_kind": "oidc",
      "provider_name": "google",
      "provider_user_id": "11335577",
      "oidc_issuer": "https://accounts.google.com",
      "oidc_subject": "sub_xxx"
    }
  ]
}
```

说明：

- 用户 `identities` 支持多 provider（如 `oidc/google`、`oauth2/github`）。
- 同一用户内，`provider_kind + provider_name` 不能重复。
- 为未来联邦登录预留了 `oidc_issuer`、`oidc_subject` 与 `metadata` 字段。

### 更新用户基本信息

`PATCH /api/v1/users/{user_id}`

支持按需更新：`username`、`display_name`、`email`、`phone`、`avatar_url`、`is_active`、`metadata`。

### 绑定外部账号

`POST /api/v1/users/{user_id}/identities`

请求体与 `CreateUserRequest.identities[]` 元素一致。

### 删除外部账号绑定

`DELETE /api/v1/users/{user_id}/identities/{identity_id}`
