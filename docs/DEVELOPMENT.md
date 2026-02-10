# 开发指南

## 1. 本地启动

```bash
cp .env.example .env
devenv shell
devenv up
task backend:dev
```

## 2. 前端开发

```bash
task frontend:dev
```

## 3. 单体运行（release）

```bash
cargo run --release
```

说明：仅 release profile 会在编译期执行前端 clean build 并嵌入二进制；debug profile 不做前端构建与嵌入。

## 4. OpenAPI（规范中心）

后端导出：

```bash
task openapi:gen
```

前端生成：

```bash
task frontend:gen:api
```

说明：`gen:api` 会同时生成 API 调用函数与 Zod schemas，前端提交前校验统一复用该 schemas。

接口调用约束检查：

```bash
task frontend:check:api-usage
```

## 5. 提交前检查

```bash
task check
```

如需包含 OpenAPI/前端生成物一致性检查：

```bash
task check:full
```

## 6. go-task 命令中心

统一脚本入口位于仓库根目录 `Taskfile.yml`，可通过以下命令查看全部任务：

```bash
task --list
```
