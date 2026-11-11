# Codex Proxy 实时请求日志插件

这是一个面向 [codex-proxy-rs](https://github.com/zyycn/codex-proxy-rs) 的可信进程插件。它只记录对话请求（`generate`）：开始时记为进行中，完成或失败后更新同一条，并统计 tok/s、输入缓存率和写入缓存率。模型列表等 `query` 请求不会进入日志。

## 在 CPR 中安装

不要直接上传 GitHub 源码目录。打开 CPR 的「插件管理 → 安装 → GitHub」，填写 `BingZi-233/cpr-plugin-request-realTime-display`，选择 `v1.0.0`，再确认安装对应平台的 Release 安装包。当前 Release 提供 macOS Apple Silicon 包。

## 特性

- 只记录 `generate` 对话请求。模型列表等查询的 operation 是 `query`，不会写入日志
- 管理页每秒刷新，展示时间、状态、密钥名称、模型、输入输出 Token、tok/s、缓存率和请求 ID
- tok/s 按输出 Token / 上游响应耗时计算；没有上游耗时时用总耗时。输入缓存率是缓存读取 / 输入，写入缓存率是缓存写入 / 输入
- 日志保留最近 200 条，不读取密钥原文或请求正文
- 页面使用宿主桥接 API，安装后无需额外 HTTP 服务

## 开发

环境要求：Rust 1.99.1、Node.js 24、pnpm 12.6。

```bash
pnpm --dir examples/workbench/frontend install --frozen-lockfile
pnpm --dir examples/workbench/frontend dev
RUST_MIN_STACK=16777216 cargo test --manifest-path examples/workbench/backend/Cargo.toml --locked
```

构建安装包：

```bash
PLUGIN_CLI="$PWD/.tools/bin/cpr-plugin" bash scripts/package
```

插件清单位于 `examples/workbench/plugin.json`，后端入口为 `examples/workbench/backend/src/app.rs`，前端入口为 `examples/workbench/frontend/src/App.vue`。
