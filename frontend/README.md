# PROJECT_NAME Frontend

最小前端：`/login` + `/settings`。

## 开发

```bash
bun install
bun run dev
```

## OpenAPI 客户端生成（可选）

```bash
bun run gen:api
```

`gen:api` 会同时生成请求函数客户端与前端表单校验用的 Zod schemas。
