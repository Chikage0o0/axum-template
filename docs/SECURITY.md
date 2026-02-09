# 安全规范（模板）

本模板只展示“工程安全约束”，不提供任何业务交易能力。

## 1. 禁止硬编码敏感信息

- 任何密钥/密码不得写入源码
- `.env` 必须被忽略（模板只提供 `.env.example`）

## 2. 密钥不回显

`GET /api/v1/settings` 不回传 `integrations.example_api_key` 明文，只回传 `example_api_key_is_set`。

## 3. 会话失效策略

- Access Token 有效期 15 分钟，Refresh Token 使用 HttpOnly Cookie（默认 30 天）
- `POST /api/v1/sessions/refresh` 会轮换 refresh token，旧 refresh token 立即失效
- 修改当前登录用户密码（`PATCH /api/v1/security/password`）会撤销该用户全部会话（所有设备需重新登录），不影响其他用户
