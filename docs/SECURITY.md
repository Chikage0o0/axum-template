# 安全规范（模板）

本模板只展示“工程安全约束”，不提供任何业务交易能力。

## 1. 禁止硬编码敏感信息

- 任何密钥/密码不得写入源码
- `.env` 必须被忽略（模板只提供 `.env.example`）

## 2. 密钥不回显

`GET /api/v1/settings` 不回传 `integrations.example_api_key` 明文，只回传 `example_api_key_is_set`。

## 3. 密钥轮换

修改管理员密码时同时轮换 JWT secret，使旧 token 立即失效。
