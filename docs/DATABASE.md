# 数据库（模板）

本模板只保留一张核心表：`system_config`。

## 表：system_config

字段：

- `key` (varchar, PK)
- `value` (jsonb)
- `description` (text, nullable)
- `updated_at` (timestamptz)

用途：

- 存储运行期配置，便于后续做 Web Settings 管理界面
- 支持热更新（写 DB 后刷新内存）
