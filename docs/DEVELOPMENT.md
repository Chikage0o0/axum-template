# 开发指南

## 1. 本地启动

```bash
cp .env.example .env
devenv shell
devenv up
cargo run
```

## 2. 前端开发

```bash
cd frontend
bun install
bun run dev
```

## 3. OpenAPI（规范中心）

后端导出：

```bash
cargo run -- --export-openapi > docs/openapi.json
```

前端生成：

```bash
cd frontend
bun run gen:openapi
bun run gen:openapi:zod
```

## 4. 提交前检查

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cd frontend && bun run check
```
