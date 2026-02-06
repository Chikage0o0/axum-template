# 前端约定（模板）

## 目标

- 演示登录 + 设置页的最小实现
- 演示统一 API 调用封装（自动带 token、401 触发登出）

## Svelte 5 约束

- 组件内状态优先使用 runes（`$state/$derived/$effect`）
- Kit 状态用 `$app/state`（禁止 `$app/stores`）
