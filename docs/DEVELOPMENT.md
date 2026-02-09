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

## 3. 单体运行（release）

```bash
cargo run --release
```

说明：仅 release profile 会在编译期执行前端 clean build 并嵌入二进制；debug profile 不做前端构建与嵌入。

## 4. OpenAPI（规范中心）

后端导出：

```bash
cargo run -- --export-openapi > docs/openapi.json
```

前端生成：

```bash
cd frontend
bun run gen:api
```

说明：`gen:api` 会同时生成 API 调用函数与 Zod schemas，前端提交前校验统一复用该 schemas。

接口调用约束检查：

```bash
cd frontend
bun run check:api-usage
```

## 5. 提交前检查

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cd frontend && bun run check
```
